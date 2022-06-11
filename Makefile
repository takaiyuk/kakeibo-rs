.PHONY: check test build run release clean

check:
	cargo c

test:
	cargo t

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
