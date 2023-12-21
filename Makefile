.PHONY: lint test run doc build-lambda deploy-lambda kick-lambda

include .env

lint:
	cargo fmt -- --check
	cargo clippy -- -D warnings

test:
	cargo test --all-features
	cargo tarpaulin --all-features

run:
	cargo run --bin kakeibo-rs

doc:
	cargo doc --no-deps --all-features --open

build-lambda:
	cargo lambda build --release --arm64 --bin kakeibo-rs-lambda

deploy-lambda:
	cargo lambda deploy kakeibo-rs-lambda --env-file .env

kick-lambda:
	cargo lambda invoke kakeibo-rs-lambda --remote --output-format json --data-ascii "{}"
