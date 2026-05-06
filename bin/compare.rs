use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).map(|s| s.to_string()).collect();
    if args.len() < 2 {
        bail!("provide paths to generated and to original files")
    }
    let generated = read_syntax(&args[0]).context("failed to read generated")?;
    let orig = read_syntax(&args[1]).context("failed to read original")?;
    similar_asserts::assert_eq!(orig, generated);
    Ok(())
}

fn read_syntax(path: &str) -> Result<Syntax> {
    if !PathBuf::from(path).exists() {
        bail!("path {path} not exists");
    }
    let s = std::fs::read_to_string(path)?;
    Ok(yaml_serde::from_str(&s)?)
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct Syntax {
    contexts: HashMap<String, Vec<Match>>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct Match {
    #[serde(rename = "match")]
    matches: String,
    captures: HashMap<u32, String>,
    embed: String,
    embed_scope: String,
}
