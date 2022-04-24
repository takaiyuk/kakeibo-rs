.PHONY: check build run clean

check:
	cargo c

build:
	cargo b

run:
	cargo r

clean:
	cargo clean

# lambda: 
# 	cargo run --release --bin lambda
