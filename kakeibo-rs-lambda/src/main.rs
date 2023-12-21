use lambda_runtime::{run, service_fn, Error, LambdaEvent};

use serde::{Deserialize, Serialize};

use kakeibo_rs::handler::run_kakeibo;

/// This is a made-up example. Requests come into the runtime as unicode
/// strings in json format, which can map to any structure that implements `serde::Deserialize`
/// The runtime pays no attention to the contents of the request payload.
// NOTE: `aws lambda invoke`` で --payload で渡すときは key になる値を Request 構造体のフィールド名にする必要がある
#[derive(Deserialize)]
#[cfg(not(tarpaulin_include))]
struct Request {}

/// This is a made-up example of what a response structure may look like.
/// There is no restriction on what it can be. The runtime requires responses
/// to be serialized into json. The runtime pays no attention
/// to the contents of the response payload.
#[derive(Serialize)]
#[cfg(not(tarpaulin_include))]
struct Response {
    req_id: String,
    msg: String,
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - <https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples>
/// - <https://github.com/aws-samples/serverless-rust-demo/>
#[cfg(not(tarpaulin_include))]
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    run_kakeibo()?;

    // Prepare the response
    let resp = Response {
        req_id: event.context.request_id,
        msg: "done".to_string(),
    };

    // Return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(resp)
}

#[tokio::main]
#[cfg(not(tarpaulin_include))]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
