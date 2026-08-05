install:
	cargo install --path .

clean:
	@fd --no-ignore --type file -e asm --max-depth 1 -x rm
	@fd --no-ignore --type file -e o -x rm
	@fd --no-ignore --type file --glob 'a.out' -x rm
	@rm -f test
