use schemars::schema::RootSchema;
use std::collections::HashMap;
type ErnieBotFunction = erniebot_rs::chat::Function;
use anyhow::Result;

use super::{
    calculator::CalculatorFunction, direct_reply::DirectReplyFunction,
    document_summary::DocumentSummaryFunction,
};

pub struct Context {
    pub session_id: i32,
    pub chat_endpoint: erniebot_rs::chat::ChatEndpoint,
}

pub trait Function {
    fn execute(&self, parameters: serde_json::Value, context: &Context) -> Result<String>;
    fn if_postprocess(&self) -> bool;
    fn get_name(&self) -> String;
    fn get_description(&self) -> String;
    fn get_parameter_schema(&self) -> RootSchema;
}

pub struct FunctionRegistry {
    functions: HashMap<String, Box<dyn Function>>,
}

unsafe impl Send for FunctionRegistry {}
unsafe impl Sync for FunctionRegistry {}

impl FunctionRegistry {
    pub fn new() -> Self {
        let functions = HashMap::new();
        Self { functions }
    }

    pub fn execute_function_by_name(
        &self,
        function_name: &str,
        parameters: serde_json::Value,
        context: &Context,
    ) -> Result<String> {
        match self.functions.get(function_name) {
            Some(function) => function.execute(parameters, context),
            None => Err(anyhow::anyhow!("Function not found")),
        }
    }

    pub fn if_postprocess_by_name(&self, function_name: &str) -> bool {
        match self.functions.get(function_name) {
            Some(function) => function.if_postprocess(),
            None => false,
        }
    }

    pub fn get_ernie_functions(&self) -> Vec<ErnieBotFunction> {
        self.functions
            .iter()
            .map(|(name, function)| ErnieBotFunction {
                name: name.clone(),
                description: function.get_description(),
                parameters: function.get_parameter_schema(),
                ..Default::default()
            })
            .collect()
    }
}

pub fn get_function_registry() -> FunctionRegistry {
    let mut registry = FunctionRegistry::new();
    registry
        .functions
        .insert("direct_reply".to_string(), Box::new(DirectReplyFunction {}));
    registry
        .functions
        .insert("calculator".to_string(), Box::new(CalculatorFunction {}));
    registry.functions.insert(
        "document_summary".to_string(),
        Box::new(DocumentSummaryFunction {}),
    );
    registry
}
