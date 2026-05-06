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
    // println!("keys, missing in orig:");
    // diff_missing(&generated.contexts, &orig.contexts);

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
    embed: String,
    embed_scope: String,
}

fn rename_map_keys<V>(map: HashMap<String, V>) -> HashMap<String, V> {
    map.into_iter().map(|(k, v)| (rename(&k), v)).collect()
}

fn rename(s: &str) -> String {
    let s = s.trim_start_matches("fenced-");
    let res = match s {
        "as" => "actionscript",
        "bat" => "dosbatch",
        "cs" => "csharp",
        "dot" => "graphviz",
        "erl" => "erlang",
        "go" => "golang",
        "json" => "jsonc",
        "jsp" => "jspx",
        "sh" => "shell-script",
        "ts" => "typescript",
        _ => s,
    };
    format!("fenced-{res}")
}

fn diff_common_keys(orig: &Matches, generated: &Matches) {
    println!("diff keys:");
    let mut keys: Vec<_> = orig.keys().collect();
    keys.sort_unstable();
    for key in keys {
        let orig_v = orig.get(key).unwrap();
        if let Some(gen_v) = generated.get(key) {
            if orig_v.len() != gen_v.len() {
                println!(
                    "  {key}: different number of matches {} != {}",
                    orig_v.len(),
                    gen_v.len()
                );
            } else if orig_v.len() != 1 {
                println!("  {key}: number of matches is not 1: {}", orig_v.len());
            } else {
                diff_match(key, &orig_v[0], &gen_v[0]);
            }
        }
    }
}

fn diff_match(key: &str, orig: &Match, generated: &Match) {
    let mut header_shown = false;
    let mut header = || {
        if !header_shown {
            println!("  ------- {key} -------");
        }
        header_shown = true;
    };

    if orig.matches != generated.matches {
        header();
        fn trim(s: &str) -> &str {
            s.trim_start_matches("(`{3,})((?i:")
                .trim_end_matches(r#"))($\n?|\b)"#)
        };
        println!(
            "    matches {}",
            diff(trim(&orig.matches), trim(&generated.matches))
        );
    }
    if orig.embed != generated.embed {
        header();
        fn trim(s: &str) -> &str {
            s.trim_start_matches("scope:")
        };
        println!(
            "    embed {}",
            diff(trim(&orig.embed), trim(&generated.embed))
        );
    }
    if orig.embed_scope != generated.embed_scope {
        header();
        println!(
            "    embed_scope {}",
            diff(&orig.embed_scope, &generated.embed_scope)
        );
    }
}

/// Print keys missing in `search` compared to `source`
fn diff_missing(source: &Matches, search: &Matches) {
    let mut missing: Vec<_> = source.keys().filter(|&k| !search.contains_key(k)).collect();
    missing.sort_unstable();
    for k in missing {
        println!("  {k}");
    }
}

fn diff(orig: &str, generated: &str) -> String {
    similar_asserts::SimpleDiff::from_str(orig, generated, "orig", "gen").to_string()
}
