.PHONY: check fmt fix test coverage build run release clean doc

check:
	cargo c

fmt:
	cargo fmt

fix:
	cargo fix

test:
	cargo t

coverage:
	cargo tarpaulin

build:
	cargo b

run:
	cargo r

release:
	cargo b --release

clean:
	cargo clean

doc:
	cargo doc --no-deps

# lambda: 
# 	cargo run --release --bin lambda
