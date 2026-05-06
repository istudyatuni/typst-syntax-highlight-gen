#![expect(unused)]

use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

type Matches = HashMap<String, Vec<Match>>;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).map(|s| s.to_string()).collect();
    if args.len() < 2 {
        bail!("provide paths to generated and to original files")
    }
    let mut generated = read_syntax(&args[0]).context("failed to read generated")?;
    generated.contexts = rename_map_keys(generated.contexts);
    let orig = read_syntax(&args[1]).context("failed to read original")?;

    diff_common_keys(&orig.contexts, &generated.contexts);
    println!("keys, missing in generated:");
    diff_missing(&orig.contexts, &generated.contexts);
    println!("keys, missing in orig:");
    diff_missing(&generated.contexts, &orig.contexts);

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
    contexts: Matches,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct Match {
    #[serde(rename = "match")]
    matches: String,
    captures: HashMap<u32, String>,
    embed: String,
    embed_scope: String,
}

fn rename_map_keys<V>(map: HashMap<String, V>) -> HashMap<String, V> {
    map.into_iter()
        .map(|(k, v)| (rename(&k).to_string(), v))
        .collect()
}

fn rename(s: &str) -> &str {
    match s {
        "as" => "actionscript",
        "bat" => "dosbatch",
        "clj" => "clojure",
        "cs" => "csharp",
        "dot" => "graphviz",
        "erl" => "erlang",
        "go" => "golang",
        "hs" => "haskell",
        "js" => "javascript",
        "make" => "makefile",
        "py" => "python",
        "tex" => "latex",
        _ => s,
    }
}

fn diff_common_keys(a: &Matches, b: &Matches) {
    println!("diff keys:");
    for (ak, av) in a {
        if let Some(bv) = b.get(ak) {
            if av.len() != b.len() {
                println!("  {ak}: different number of matches");
            } else if av.len() != 1 {
                println!("  {ak}: number of matches is not 1: {}", av.len());
            } else {
                println!("  {ak}:");
                diff_match(&av[0], &bv[0]);
            }
        }
    }
}

fn diff_match(a: &Match, b: &Match) {
    if a.matches != b.matches {
        println!("    {}", diff(&a.matches, &b.matches));
    }
}

/// Print keys missing in `search` compared to `source`
fn diff_missing(source: &Matches, search: &Matches) {
    for (k, v) in source {
        if search.get(k).is_none() {
            println!("  {k}");
        }
    }
}

fn diff(a: &str, b: &str) -> String {
    similar_asserts::SimpleDiff::from_str(a, b, "left", "right").to_string()
}
