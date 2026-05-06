[private]
@default:
	just list --unsorted

# build syntax
build *args:
	cargo r --bin=gen -- {{ args }} > typst-fenced.sublime-syntax

# compare generated and original syntax
compare orig: (build "--skip-main")
	cargo r --bin=compare -- typst-fenced.sublime-syntax '{{ orig }}'
