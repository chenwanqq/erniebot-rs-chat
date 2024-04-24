use crate::entities;
use anyhow::Result;
use erniebot_rs::chat::{ChatEndpoint, ChatOpt, Message, Role};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::functions::{get_function_registry, Context, FunctionRegistry};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionSelectResult {
    function: String,
    parameters: serde_json::Value,
    thoughts: String,
}

fn generate_request(message: &str, function_registry: &FunctionRegistry) -> Result<String> {
    //从templates/select.template读取
    let template = fs::read_to_string("templates/select.template")?;
    let function_list = function_registry.get_ernie_functions();
    let functions_string = serde_json::to_string(&function_list)?;
    let request = template
        .replace("{{functions}}", &functions_string)
        .replace("{{message}}", message);
    Ok(request)
}

fn extract_result(response_string: &str) -> Result<FunctionSelectResult> {
    let result = response_string;
    let mut start_line: i32 = -1;
    let mut end_line: i32 = -1;
    for (cnt, line) in result.lines().enumerate() {
        if line.starts_with('{') && start_line == -1 {
            start_line = cnt as i32;
        }
        if line.starts_with('}') {
            end_line = cnt as i32;
        }
    }
    let result = result
        .lines()
        .skip((start_line) as usize)
        .take((end_line - start_line + 1) as usize)
        .collect::<Vec<&str>>()
        .join("\n");
    let result = serde_json::from_str(&result)?;
    Ok(result)
}

async fn try_get_response(
    chat_endpoint: &ChatEndpoint,
    messages: &Vec<Message>,
    options: &Vec<ChatOpt>,
) -> Result<String> {
    let response = chat_endpoint.ainvoke(messages, options).await?;
    let result = response.get_chat_result()?;
    Ok(result)
}

async fn select_function(
    message: &str,
    chat_history: &mut Vec<Message>,
    chat_endpoint: &ChatEndpoint,
    function_registry: &FunctionRegistry,
    max_retry: usize,
) -> Result<FunctionSelectResult> {
    let request = generate_request(message, function_registry)?;
    chat_history.push(Message {
        role: Role::User,
        content: request,
        ..Default::default()
    });

    let options = Vec::new();
    let mut retry = 0;
    loop {
        let try_result = try_get_response(chat_endpoint, chat_history, &options).await;
        if try_result.is_err() {
            println!("{:?}", try_result);
            retry += 1;
            if retry >= max_retry {
                return Err(anyhow::anyhow!("Max retry reached"));
            }
            continue;
        }
        let response = try_result.unwrap();
        chat_history.push(Message {
            role: Role::Assistant,
            content: response.clone(),
            ..Default::default()
        });
        match extract_result(&response) {
            Ok(result) => return Ok(result),
            Err(_) => {
                retry += 1;
                if retry >= max_retry {
                    return Err(anyhow::anyhow!("Max retry reached"));
                } else {
                    chat_history.push(Message {
                        role: Role::User,
                        content: "辛苦您了，但是您的回复中似乎没有我想要的符合要求的json。请您重新尝试回答刚才的问题".to_string(),
                        ..Default::default()
                    });
                }
            }
        }
    }
}

async fn postprocess(
    response: &str,
    chat_history: &mut Vec<Message>,
    chat_endpoint: &ChatEndpoint,
) -> Result<String> {
    let template = fs::read_to_string("templates/postprocess.template")?;
    let request = template.replace("{{response}}", response);
    let options = Vec::new();
    chat_history.push(Message {
        role: Role::User,
        content: request,
        ..Default::default()
    });
    let response = try_get_response(chat_endpoint, chat_history, &options).await?;
    Ok(response)
}

async fn fallback(
    message: &str,
    chat_history: &mut Vec<Message>,
    chat_endpoint: &ChatEndpoint,
) -> Result<String> {
    let options = Vec::new();
    chat_history.push(Message {
        role: Role::User,
        content: message.to_string(),
        ..Default::default()
    });
    let response = try_get_response(chat_endpoint, chat_history, &options).await?;
    Ok(response)
}

async fn single_step(
    message: &str,
    chat_history: &mut Vec<Message>,
    chat_endpoint: &ChatEndpoint,
    function_registry: &FunctionRegistry,
    max_retry: usize,
    context: &Context,
) -> Result<String> {
    let original_history_length = chat_history.len();
    let result = select_function(
        message,
        chat_history,
        chat_endpoint,
        function_registry,
        max_retry,
    )
    .await?;
    println!("function choice: {:?}", result);
    let function_name = result.function;
    let parameters = result.parameters;
    let response = function_registry.execute_function_by_name(&function_name, parameters, context);
    let response = match response {
        Ok(response) => {
            if function_registry.if_postprocess_by_name(&function_name) {
                postprocess(&response, chat_history, chat_endpoint).await?
            } else {
                response
            }
        }
        Err(_) => {
            chat_history.truncate(original_history_length);
            println!("fallback");
            fallback(message, chat_history, chat_endpoint).await?
        }
    };
    println!("response: {:?}", response);
    Ok(response)
}

fn role_transform(sea_role: &entities::sea_orm_active_enums::Role) -> Role {
    match sea_role {
        crate::sea_orm_active_enums::Role::User => Role::User,
        crate::sea_orm_active_enums::Role::Assistant => Role::Assistant,
        crate::sea_orm_active_enums::Role::Function => Role::Function,
    }
}

pub async fn reply(
    message: &str,
    session_id: i32,
    db: &DatabaseConnection,
    chat_endpoint: &ChatEndpoint,
) -> Result<String> {
    let context = Context {
        session_id,
        chat_endpoint: chat_endpoint.clone(),
    };
    let function_registry = get_function_registry();
    let user_message_time = chrono::Utc::now();
    let mut messages = entities::prelude::Message::find()
        .filter(entities::message::Column::SessionId.eq(session_id))
        .all(db)
        .await?;
    messages.sort_by_key(|x| x.create_time);
    let mut chat_history: Vec<Message> = messages
        .iter()
        .map(|x| Message {
            role: role_transform(&x.role),
            content: x.content.clone(),
            ..Default::default()
        })
        .collect();
    let response = single_step(
        message,
        &mut chat_history,
        chat_endpoint,
        &function_registry,
        3,
        &context,
    )
    .await?;
    let user_message = entities::message::ActiveModel {
        session_id: Set(session_id),
        role: Set(entities::sea_orm_active_enums::Role::User),
        content: Set(message.to_string()),
        message_type: Set(entities::sea_orm_active_enums::MessageType::Text),
        create_time: Set(user_message_time),
        ..Default::default()
    };
    let assistant_message = entities::message::ActiveModel {
        session_id: Set(session_id),
        role: Set(entities::sea_orm_active_enums::Role::Assistant),
        content: Set(response.clone()),
        message_type: Set(entities::sea_orm_active_enums::MessageType::Text),
        create_time: Set(chrono::Utc::now()),
        ..Default::default()
    };
    entities::prelude::Message::insert(user_message)
        .exec(db)
        .await?;
    entities::prelude::Message::insert(assistant_message)
        .exec(db)
        .await?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use crate::functions::get_function_registry;
    #[test]
    fn test_generate_request() {
        let message = "你好".to_string();
        let function_registry = get_function_registry();
        let request = super::generate_request(&message, &function_registry).unwrap();
        println!("{}", request);
    }
}
