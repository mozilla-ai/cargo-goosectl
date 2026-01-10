use std::{fs::File, io::Write, path::PathBuf};

use anyhow::Result;
use schemars::{JsonSchema, schema_for};

const SCHEMA_PATH: &str = "schemas";

fn main() -> Result<()> {
    write_to_path::<cargo_goose::config::Config>("goose")?;
    Ok(())
}

fn write_to_path<T: JsonSchema>(name: &str) -> Result<()> {
    let schema = schema_for!(T);
    let schema_bytes = serde_json::to_vec_pretty(&schema)?;

    let path = PathBuf::from(SCHEMA_PATH).join(format!("{name}.schema.json"));
    let mut file = File::create(&path)?;
    file.write_all(&schema_bytes)?;

    Ok(())
}
