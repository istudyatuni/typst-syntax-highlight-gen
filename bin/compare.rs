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
    captures: HashMap<u32, String>,
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
        "clj" => "clojure",
        "cs" => "csharp",
        "dot" => "graphviz",
        "erl" => "erlang",
        "go" => "golang",
        "hs" => "haskell",
        "js" => "javascript",
        "json" => "jsonc",
        "jsp" => "jspx",
        "make" => "makefile",
        "ml" => "ocaml",
        "pl" => "perl",
        "py" => "python",
        "rb" => "ruby",
        "re" => "regexp",
        "rs" => "rust",
        "sh" => "shell-script",
        "tex" => "latex",
        "ts" => "typescript",
        _ => s,
    };
    format!("fenced-{res}")
}

fn diff_common_keys(a: &Matches, b: &Matches) {
    println!("diff keys:");
    let mut keys: Vec<_> = a.keys().collect();
    keys.sort_unstable();
    for key in keys {
        let av = a.get(key).unwrap();
        if let Some(bv) = b.get(key) {
            if av.len() != bv.len() {
                println!(
                    "  {key}: different number of matches {} != {}",
                    av.len(),
                    bv.len()
                );
            } else if av.len() != 1 {
                println!("  {key}: number of matches is not 1: {}", av.len());
            } else {
                diff_match(key, &av[0], &bv[0]);
            }
        }
    }
}

fn diff_match(key: &str, a: &Match, b: &Match) {
    let mut header_shown = false;
    let mut header = || {
        if !header_shown {
            println!("  {key}:");
        }
        header_shown = true;
    };

    if a.matches != b.matches {
        header();
        fn trim(s: &str) -> &str {
            s.trim_start_matches("(`{3,})((?i:")
                .trim_end_matches(r#"))($\n?|\b)"#)
        };
        println!("    {}", diff(trim(&a.matches), trim(&b.matches)));
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

fn diff(a: &str, b: &str) -> String {
    similar_asserts::SimpleDiff::from_str(a, b, "orig", "gen").to_string()
}
