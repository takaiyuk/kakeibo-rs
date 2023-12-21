.PHONY: lint test run build-lambda doc

include .env

lint:
	cargo fmt -- --check
	cargo clippy -- -D warnings

test:
	cargo test --all-features
	cargo tarpaulin --all-features

run:
	cargo run --bin kakeibo-rs

build-lambda:
	docker build -t kakeibo-rs-lambda -f ./docker/lambda/Dockerfile .
	docker run --rm -v $(PWD)/target:/workspace/target kakeibo-rs-lambda cargo lambda build --release --arm64 --bin lambda --output-format zip

deploy-lambda:
	aws lambda create-function --function-name kakeibo-rs \
		--handler bootstrap \
		--zip-file fileb://./target/lambda/lambda/bootstrap.zip \
		--runtime provided.al2023 \
		--role $(LAMBDA_ROLE_ARN) \
		--environment Variables="{RUST_BACKTRACE=1,IFTTT_EVENT_NAME=$(IFTTT_EVENT_NAME),IFTTT_WEBHOOK_TOKEN=$(IFTTT_WEBHOOK_TOKEN),SLACK_TOKEN=$(SLACK_TOKEN),SLACK_CHANNEL_ID=$(SLACK_CHANNEL_ID)}" \
		--tracing-config Mode=Active \
		--architectures arm64

update-lambda-code:
	aws lambda update-function-code --function-name kakeibo-rs \
		--zip-file fileb://./target/lambda/lambda/bootstrap.zip

update-lambda-configuration:
	aws lambda update-function-configuration --function-name kakeibo-rs \
		--environment Variables="{RUST_BACKTRACE=1,IFTTT_EVENT_NAME=$(IFTTT_EVENT_NAME),IFTTT_WEBHOOK_TOKEN=$(IFTTT_WEBHOOK_TOKEN),SLACK_TOKEN=$(SLACK_TOKEN),SLACK_CHANNEL_ID=$(SLACK_CHANNEL_ID)}"

kick-lambda:
	aws lambda invoke --cli-binary-format raw-in-base64-out --function-name kakeibo-rs /dev/stdout

doc:
	cargo doc --no-deps --all-features --open
