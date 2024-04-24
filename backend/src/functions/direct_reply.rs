use super::function::{Context, Function};
use anyhow::Result;
use schemars::{schema::RootSchema, schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, PartialEq)]
struct DirectReplyParameters {
    message: String,
}
pub struct DirectReplyFunction {}

impl Function for DirectReplyFunction {
    fn execute(&self, parameters: serde_json::Value, _: &Context) -> Result<String> {
        let parameters: DirectReplyParameters = serde_json::from_value(parameters)?;
        Ok(parameters.message)
    }

    fn if_postprocess(&self) -> bool {
        false
    }

    fn get_name(&self) -> String {
        "direct_reply".to_string()
    }

    fn get_description(&self) -> String {
        "当用户的问题不涉及专业问题，只是想聊天时,或者没有别的更合适的函数时，一个大模型可以将想回复的信息直接输入进来作为回复输出。".to_string()
    }

    fn get_parameter_schema(&self) -> RootSchema {
        schema_for!(DirectReplyParameters)
    }
}
