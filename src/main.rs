#![expect(unused)]

use typst_library::text::RawElem;

const HEADER: &str = r#"
%YAML 1.2
---
contexts:
  main:
    - include: markups
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

fn main() {
    println!("{}", HEADER.trim());

    let mut is_native = 0;
    for (name, exts) in RawElem::languages() {
        if exts.is_empty() {
            continue;
        }
        if name == "Typst" && exts == ["typ"] || is_native > 0 {
            is_native += 1;
            continue;
        }
        let comment = format!("{name}: {}", exts.join(", "));
        let exts: Vec<_> = exts
            .into_iter()
            .map(ext_to_tag)
            .collect();
        let res = fill_template(name, &exts, &comment);
        println!("{res}");
    }
    assert_eq!(is_native, 3);
}

fn fill_template(name: &str, exts: &[String], comment: &str) -> String {
    let scope = ext_to_tag(&exts[0]);
    // todo: also use custom scopes
    let full_scope = format!("scope.{scope}");
    let matches = exts
        .iter()
        .map(|e| ext_to_tag(e))
        .collect::<Vec<_>>()
        .join("|");
    TEMPLATE
        .replace("$SCOPE", &scope)
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
    ext.trim_start_matches(".").to_string()
}
