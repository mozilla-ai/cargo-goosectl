use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Rust,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct LanguageConfig {}
