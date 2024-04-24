use super::function::{Context, Function};
use anyhow::Result;
use evalexpr::eval;
use schemars::{schema::RootSchema, schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, PartialEq)]
struct CalculatorParameters {
    expression: String,
}

pub struct CalculatorFunction {}
impl Function for CalculatorFunction {
    fn execute(&self, parameters: serde_json::Value, _: &Context) -> Result<String> {
        let parameters: CalculatorParameters = serde_json::from_value(parameters)?;
        let expression = parameters.expression;
        let result = eval(&expression)?;
        Ok(result.to_string())
    }

    fn if_postprocess(&self) -> bool {
        true
    }

    fn get_name(&self) -> String {
        "calculator".to_string()
    }

    fn get_description(&self) -> String {
        "涉及到简单数学问题时，可以将问题表述为一个非代数表达式，使用该函数来计算该表达式的值。请注意该函数的限制，如果不能表达为一个数学表达式，请不要使用该函数".to_string()
    }

    fn get_parameter_schema(&self) -> RootSchema {
        schema_for!(CalculatorParameters)
    }
}
