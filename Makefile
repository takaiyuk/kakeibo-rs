.PHONY: check build run release clean

check:
	cargo c

build:
	cargo b

run:
	cargo r

release:
	cargo build --release

clean:
	cargo clean

# lambda: 
# 	cargo run --release --bin lambda
