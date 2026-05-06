#![expect(unused)]

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

type Matches = HashMap<String, Vec<Match>>;
type Ignores = BTreeMap<String, Ignore>;

const IGNORE_FILE: &str = "ignore.json";

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).map(|s| s.to_string()).collect();
    if args.len() < 2 {
        bail!("provide paths to generated and to original files")
    }
    let mut generated = read_syntax(&args[0]).context("failed to read generated")?;
    generated.contexts = rename_map_keys(generated.contexts);
    let orig = read_syntax(&args[1]).context("failed to read original")?;

    let ignores = std::fs::read_to_string(IGNORE_FILE).context("failed to read ignore data")?;
    let ignores: Ignores = serde_json::from_str(&ignores).context("failed to parse ignore data")?;

    diff_common_keys(&orig.contexts, &generated.contexts, &ignores);
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

fn diff_common_keys(orig: &Matches, generated: &Matches, ignores: &Ignores) {
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
                diff_match(
                    key,
                    &orig_v[0],
                    &gen_v[0],
                    ignores.get(key.trim_start_matches("fenced-")),
                );
            }
        }
    }
}

fn diff_match(key: &str, orig: &Match, generated: &Match, ignore: Option<&Ignore>) {
    let mut header_shown = false;
    let mut header = || {
        if !header_shown {
            println!("  ------- {key} -------");
        }
        header_shown = true;
    };

    'b: {
        if orig.matches != generated.matches {
            fn trim(s: &str) -> &str {
                s.trim_start_matches("(`{3,})((?i:")
                    .trim_end_matches(r#"))($\n?|\b)"#)
            };
            // ignore elements present in both
            let orig: HashSet<_> = trim(&orig.matches).split("|").collect();
            let generated: HashSet<_> = trim(&generated.matches).split("|").collect();
            let common: HashSet<_> = orig.intersection(&generated).copied().collect();
            let mut orig = orig.difference(&common).copied().collect::<Vec<_>>();
            orig.sort_unstable();
            if orig.is_empty() {
                break 'b;
            }

            let mut generated = generated.difference(&common).copied().collect::<Vec<_>>();
            generated.sort_unstable();

            let orig = orig.join(",");
            let generated = generated.join(",");

            if let Some(ignore) = ignore
                && let Some(ignore) = &ignore.matches
                && ignore.matches(&orig, &generated)
            {
                break 'b;
            }

            header();
            println!("    matches {}", diff(&orig, &generated).trim());
            if !common.is_empty() {
                println!("    {} not changed\n", common.len());
            }
        }
    }
    'b: {
        if orig.embed != generated.embed {
            fn trim(s: &str) -> &str {
                s.trim_start_matches("scope:")
            };

            let orig = trim(&orig.embed);
            let generated = trim(&generated.embed);
            if let Some(ignore) = ignore
                && let Some(ignore) = &ignore.embed
                && ignore.matches(orig, generated)
            {
                break 'b;
            }

            header();
            println!("    embed {}", diff(orig, generated));
        }
    }
    'b: {
        if orig.embed_scope != generated.embed_scope {
            let orig = &orig.embed_scope;
            let generated = &generated.embed_scope;
            if let Some(ignore) = ignore
                && let Some(ignore) = &ignore.embed_scope
                && ignore.matches(orig, generated)
            {
                break 'b;
            }

            header();
            println!("    embed_scope {}", diff(orig, generated));
        }
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

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct Ignore {
    matches: Option<Diff>,
    embed: Option<Diff>,
    embed_scope: Option<Diff>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Diff {
    del: String,
    add: String,
}

impl Diff {
    fn matches(&self, del: &str, add: &str) -> bool {
        self.del == del && self.add == add
    }
}
