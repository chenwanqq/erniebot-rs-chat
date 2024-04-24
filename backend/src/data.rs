use serde::{Deserialize, Serialize};
use serde_json::value::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct TextJsonResponse {
    pub code: u32,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PureCodeResponse {
    pub code: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonDataResponse {
    pub code: u32,
    pub data: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextRequest {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatRequest {
    pub session_id: i32,
    pub content: String,
    pub content_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateSessionRequest {
    pub user_id: i32,
}
