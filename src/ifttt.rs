use crate::slack::SlackMessage;
use std::collections::HashMap;
use std::env;

const IFTTT_BASE_URL: &str = "https://maker.ifttt.com/trigger";

struct IFTTTAPIParams {
    event_name: String,
    token: String,
}

pub fn kick_ifttt_webhook(slack_messages: Vec<SlackMessage>) {
    let ifttt_api_params = get_ifttt_api_params();
    post_ifttt_webhook(slack_messages, &ifttt_api_params);
}

fn get_ifttt_api_params() -> IFTTTAPIParams {
    let ifttt_api_params = IFTTTAPIParams {
        event_name: env::var("IFTTT_EVENT_NAME").expect("$IFTTT_EVENT_NAME is not set"),
        token: env::var("IFTTT_WEBHOOK_TOKEN").expect("$IFTTT_WEBHOOK_TOKEN is not set"),
    };
    ifttt_api_params
}

fn post_ifttt_webhook(slack_messages: Vec<SlackMessage>, params: &IFTTTAPIParams) {
    let ifttt_url = format!(
        "{}/{}/with/key/{}",
        IFTTT_BASE_URL, params.event_name, params.token
    );
    let ifttt_client = reqwest::blocking::Client::new();
    for m in slack_messages {
        let mut payload = HashMap::new();
        payload.insert("value1", m.timestamp.to_string());
        payload.insert("value2", m.text.to_string());
        let payload = serde_json::to_string(&payload).unwrap();
        let res = ifttt_client
            .post(&ifttt_url)
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .expect("failed to post");
        if res.status().is_success() {
            println!("message posted: {},{}", m.timestamp, m.text);
        } else {
            println!("failed to post: {},{}", m.timestamp, m.text);
        }
    }
}
