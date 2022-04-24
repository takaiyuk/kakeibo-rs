use chrono::{DateTime, Duration, Local};
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;

const SLACK_BASE_URL: &str = "https://slack.com/api";
const IFTTT_BASE_URL: &str = "https://maker.ifttt.com/trigger";

struct SlackMessage {
    timestamp: f64,
    text: String,
}

struct FilterSlackMessageOptions {
    local_dt: DateTime<Local>,
    exclude_days: i64,
    exclude_hours: i64,
    exclude_minutes: i64,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    dotenv().ok();

    let slack_api_method = "conversations.history";
    let slack_token = env::var("SLACK_TOKEN").expect("$SLACK_TOKEN is not set");
    let slack_channel_id = env::var("SLACK_CHANNEL_ID").expect("$SLACK_CHANNEL_ID is not set");
    let slack_url = format!(
        "{}/{}?channel={}",
        SLACK_BASE_URL, slack_api_method, slack_channel_id
    );
    let slack_header_auth = format!("Bearer {}", slack_token);
    let slack_client = reqwest::Client::new();
    let res: serde_json::Value = slack_client
        .post(slack_url)
        .header("Authorization", slack_header_auth)
        .send()
        .await
        .expect("failed to get response")
        .json()
        .await
        .expect("failed to get payload");
    let mut slack_messages = Vec::new();
    for i in (0..100).rev() {
        let message = &res["messages"][i];
        let ts = message["ts"].as_str().unwrap();
        let text = message["text"].as_str().unwrap();
        let slack_message = SlackMessage {
            timestamp: ts.parse::<f64>().unwrap(),
            text: text.to_string(),
        };
        slack_messages.push(slack_message);
    }

    let fiter_options = FilterSlackMessageOptions {
        local_dt: Local::now(),
        exclude_days: 0,
        exclude_hours: 0,
        exclude_minutes: 10,
    };
    let threshold: f64 = (fiter_options.local_dt
        - Duration::days(fiter_options.exclude_days)
        - Duration::hours(fiter_options.exclude_hours)
        - Duration::minutes(fiter_options.exclude_minutes))
    .timestamp() as f64;
    let mut filtered_slack_messages = Vec::new();
    for m in slack_messages.iter() {
        if m.timestamp > threshold {
            filtered_slack_messages.push(m);
            println!("message to be posted: {},{}", m.timestamp, m.text);
        }
    }

    let ifttt_event_name = env::var("IFTTT_EVENT_NAME").expect("$IFTTT_EVENT_NAME is not set");
    let ifttt_api_key = env::var("IFTTT_WEBHOOK_TOKEN").expect("$IFTTT_WEBHOOK_TOKEN is not set");
    let ifttt_url = format!(
        "{}/{}/with/key/{}",
        IFTTT_BASE_URL, ifttt_event_name, ifttt_api_key
    );
    let ifttt_client = reqwest::Client::new();
    for m in filtered_slack_messages.iter() {
        let mut payload = HashMap::new();
        payload.insert("value1", m.timestamp.to_string());
        payload.insert("value2", m.text.to_string());
        let payload = serde_json::to_string(&payload).unwrap();
        let res = ifttt_client
            .post(&ifttt_url)
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .await
            .expect("failed to post");
        if res.status().is_success() {
            println!("message posted: {},{}", m.timestamp, m.text);
        } else {
            println!("failed to post: {},{}", m.timestamp, m.text);
        }
    }

    Ok(())
}
