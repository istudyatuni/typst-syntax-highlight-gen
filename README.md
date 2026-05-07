# Sublime syntax generator for language embeddings in Typst

This script generates syntaxes for all languages supported in Typst by default

## Usage

Required: `cargo` (Rust), `just` (optional)

```bash
# build syntax definition
just build

# compare with original
# this script was used for migrating only
just compare
```

See what's beeing run in [.justfile](./.justfile) 

After generation can be copy-pasted to [package](https://github.com/hyrious/typst-syntax-highlight)

## Versions

- Tag `pr-69` points to version which was vendored in [typst-syntax-highlight #69](https://github.com/hyrious/typst-syntax-highlight/pull/69)

## Customize

### Extend generation

Extends should be added to `extend.json` in format:

```jsonc
{
	// other ...
	"clj": {
		"matches": [ "clojure" ],
		"scope": "source.clojure",
		"rename": "clojure"
	}
}
```

All fields are optional. Key is autodetected by name/extension

- `matches` - add extra patterns to `match`
- `scope` - override scope, default is `source.[key]`
- `rename` - changes key/scope, added to `matches`

### Ignore

Only for `compare`. See examples in `ignore.json`
