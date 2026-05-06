#![expect(unused)]

use std::collections::HashSet;

use typst_library::text::RawElem;

const HEADER: &str = r#"
%YAML 1.2
---
contexts:
"#;

const TEMPLATE: &str = r#"
  # $COMMENT
  fenced-$SCOPE:
    - match: (`{3,})((?i:$MATCH))($\n?|\b)
      captures:
        1: punctuation.definition.raw.code-fence.begin.typst
        2: constant.other.language-name.typst
      embed: scope:$FULL_SCOPE
      embed_scope: markup.raw.block.$SCOPE.typst source.$SCOPE
      escape: '(?:^[ \t]*)?(\1)\s*'
      escape_captures:
        1: punctuation.definition.raw.code-fence.end.typst
"#;

const SKIP_SYNTAX: &[&str] = &["txt"];

fn main() {
    println!("{}", HEADER.trim());

    let mut is_native = 0;
    let mut added_scopes = HashSet::new();
    for (name, exts) in RawElem::languages() {
        if exts.is_empty() {
            continue;
        }
        if name == "Typst" && exts == ["typ"] || is_native > 0 {
            is_native += 1;
            continue;
        }
        let comment = format!("{name}: {}", exts.join(", "));
        let exts: Vec<_> = exts.into_iter().map(ext_to_tag).collect();
        let scope = ext_to_tag(&exts[0]);
        if SKIP_SYNTAX.contains(&scope.as_str()) {
            eprintln!("skipping ignored {comment}");
            continue;
        }
        if added_scopes.contains(&scope) {
            eprintln!("skipping already added {comment}");
            continue;
        }

        let exts: Vec<_> = exts
            .into_iter()
            // deduplicate
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let res = fill_template(name, &exts, &scope, &comment);
        println!("{res}");

        added_scopes.insert(scope);
    }
    assert_eq!(is_native, 3);

    println!("  main:");
    let mut added_scopes: Vec<_> = added_scopes.iter().collect();
    added_scopes.sort_unstable();
    for scope in added_scopes {
        println!("    - include: fenced-{scope}");
    }
}

fn fill_template(name: &str, exts: &[String], scope: &str, comment: &str) -> String {
    // todo: also use custom scopes
    let full_scope = format!("scope.{scope}");
    let matches = exts
        .iter()
        .map(|e| ext_to_tag(e))
        .collect::<Vec<_>>()
        .join("|");
    TEMPLATE
        .trim_end()
        .replace("$SCOPE", scope)
        .replace("$FULL_SCOPE", &full_scope)
        .replace("$MATCH", &escape_regex(&matches))
        .replace("$COMMENT", comment)
}

fn escape_regex(s: &str) -> String {
    let mut res = s.to_string();
    for pat in [".", "+"] {
        res = res.replace(pat, &format!("\\{pat}"));
    }
    res
}

fn ext_to_tag(ext: &str) -> String {
    let ext = ext.trim_start_matches(".");
    if let Some((first, _)) = ext.split_once(".") {
        return first.to_string();
    }
    ext.to_string()
}
