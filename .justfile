[private]
@default:
	just list --unsorted

# build syntax
build:
	cargo r --bin=gen > typst-fenced.sublime-syntax

# build generated and original syntax
compare orig:
	cargo r --bin=compare -- typst-fenced.sublime-syntax '{{ orig }}'
