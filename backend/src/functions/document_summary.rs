use super::function::{Context, Function};
use anyhow::Result;
use erniebot_rs::chat::{ChatEndpoint, Message, Role};
use schemars::{schema::RootSchema, schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, PartialEq)]
struct DocumentSummaryFunctionParameters {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct DocumentSummaryConfig {
    max_chunk_length: i32,
    suggest_summary_length: i32,
}

pub struct DocumentSummaryFunction {}

impl DocumentSummaryFunction {
    fn get_documents(&self, session_id: i32) -> Result<String> {
        let filepath = format!("./files/{}.txt", session_id);
        let res = std::fs::read_to_string(filepath)?;
        Ok(res)
    }
    //逐步生成摘要
    fn get_summary(&self, chat_endpoint: &ChatEndpoint, documents: &str) -> Result<String> {
        let summary_template = std::fs::read_to_string("templates/summary.template")?;
        println!("summary_template: {:?}", summary_template);
        let summary_config_string = std::fs::read_to_string("configs/summary_config.json")?;
        println!("summary_config_string: {:?}", summary_config_string);
        let summary_config: DocumentSummaryConfig = serde_json::from_str(&summary_config_string)?;

        println!("summary_config: {:?}", summary_config);
        let chars_vec: Vec<char> = documents.chars().collect();
        let chars_len = chars_vec.len();
        println!("Document length: {}", chars_len);
        let mut previous_summary = String::new();
        let mut start = 0;
        while start < chars_len {
            let mut segment_end = start + summary_config.max_chunk_length as usize;
            if segment_end > chars_len {
                segment_end = chars_len;
            }
            let segment = chars_vec[start..segment_end].iter().collect::<String>();
            let request_string = summary_template
                .replace(
                    "{{suggest_summary_length}}",
                    &summary_config.suggest_summary_length.to_string(),
                )
                .replace("{{previous_summary}}", &previous_summary)
                .replace("{{current_text}}", &segment);

            let message = Message {
                role: Role::User,
                content: request_string,
                ..Default::default()
            };
            let options = Vec::new();
            println!("Request: {:?}", message.content);
            let response = chat_endpoint.invoke(&vec![message], &options)?;
            previous_summary = response.get_chat_result()?.to_string();
            start += summary_config.max_chunk_length as usize;
        }
        Ok(previous_summary)
    }
}

impl Function for DocumentSummaryFunction {
    fn execute(&self, _: serde_json::Value, context: &Context) -> Result<String> {
        println!("开始执行文档摘要函数");
        let session_id = context.session_id;
        let documents = self.get_documents(session_id)?;
        let summary = self.get_summary(&context.chat_endpoint, &documents)?;
        Ok(summary.to_string())
    }

    fn if_postprocess(&self) -> bool {
        true
    }

    fn get_name(&self) -> String {
        "document_summary".to_string()
    }

    fn get_description(&self) -> String {
        "当用户的问题想要生成文档摘要或者你判断问题可以从文档摘要中获得启发，可以调用该函数。"
            .to_string()
    }

    fn get_parameter_schema(&self) -> RootSchema {
        schema_for!(DocumentSummaryFunctionParameters)
    }
}
