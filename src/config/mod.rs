use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Deserialize;

mod language;

pub use language::{Language, LanguageConfig};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Config {
    #[serde(default)]
    version: GooseConfigVersion,

    #[serde(default)]
    project: GooseConfig,

    #[serde(flatten, default)]
    language: HashMap<Language, LanguageConfig>,
}

impl Config {
    pub fn version(&self) -> u64 {
        self.version.0
    }

    pub fn project(&self) -> &GooseConfig {
        &self.project
    }

    pub fn langauge(&self) -> &HashMap<Language, LanguageConfig> {
        &self.language
    }
}

#[derive(Debug, Deserialize, Default, JsonSchema)]
pub struct GooseConfig {}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(transparent)]
struct GooseConfigVersion(u64);

impl Default for GooseConfigVersion {
    fn default() -> Self {
        Self(1)
    }
}
