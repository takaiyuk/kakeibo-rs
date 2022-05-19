use crate::slack::SlackMessage;
use std::collections::HashMap;
use std::env;

const IFTTT_BASE_URL: &str = "https://maker.ifttt.com/trigger";

pub struct IFTTTAPIParams {
    event_name: String,
    token: String,
}

impl IFTTTAPIParams {
    pub fn build() -> IFTTTAPIParams {
        IFTTTAPIParams {
            event_name: env::var("IFTTT_EVENT_NAME").expect("$IFTTT_EVENT_NAME is not set"),
            token: env::var("IFTTT_WEBHOOK_TOKEN").expect("$IFTTT_WEBHOOK_TOKEN is not set"),
        }
    }
}

pub struct IFTTTAPI {
    pub params: IFTTTAPIParams,
}

impl IFTTTAPI {
    pub fn kick(&self, slack_messages: Vec<SlackMessage>) {
        self.post_ifttt_webhook(slack_messages);
    }

    fn post_ifttt_webhook(&self, slack_messages: Vec<SlackMessage>) {
        let ifttt_url = format!(
            "{}/{}/with/key/{}",
            IFTTT_BASE_URL, self.params.event_name, self.params.token
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
}
