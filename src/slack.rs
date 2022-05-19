use chrono::{DateTime, Duration, Local};
use std::env;

const SLACK_BASE_URL: &str = "https://slack.com/api";
const SLACK_API_METHOD: &str = "conversations.history";
const EXCLUDE_DAYS: i64 = 0;
const EXCLUDE_HOURS: i64 = 0;
const EXCLUDE_MINUTES: i64 = 10;

pub struct SlackMessage {
    pub timestamp: f64,
    pub text: String,
}

pub struct SlackAPIParams {
    method: String,
    channel: String,
    token: String,
}

impl SlackAPIParams {
    pub fn build() -> SlackAPIParams {
        SlackAPIParams {
            method: SLACK_API_METHOD.to_string(),
            channel: env::var("SLACK_CHANNEL_ID")
                .expect("$SLACK_CHANNEL_ID is not set")
                .to_string(),
            token: env::var("SLACK_TOKEN")
                .expect("$SLACK_TOKEN is not set")
                .to_string(),
        }
    }
}

pub struct SlackAPI {
    pub params: SlackAPIParams,
}

impl SlackAPI {
    pub fn extract(&self) -> Vec<SlackMessage> {
        let local_dt = Local::now();

        let slack_messages = self.get_conversations_history();
        let fiter_options = FilterSlackMessageOptions::build(
            local_dt,
            EXCLUDE_DAYS,
            EXCLUDE_HOURS,
            EXCLUDE_MINUTES,
        );
        let threshold = fiter_options.get_filter_threshold();
        let filtered_slack_messages = self.filter(slack_messages, threshold);
        filtered_slack_messages
    }

    fn get_conversations_history(&self) -> Vec<SlackMessage> {
        let slack_url = format!(
            "{}/{}?channel={}",
            SLACK_BASE_URL, self.params.method, self.params.channel
        );
        let slack_header_auth = format!("Bearer {}", self.params.token);

        let slack_client = reqwest::blocking::Client::new();
        let res: serde_json::Value = slack_client
            .post(slack_url)
            .header("Authorization", slack_header_auth)
            .send()
            .expect("failed to get response")
            .json()
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
        slack_messages
    }

    fn filter(&self, slack_messages: Vec<SlackMessage>, threshold: f64) -> Vec<SlackMessage> {
        let mut filtered_slack_messages = Vec::new();
        for m in slack_messages {
            if m.timestamp > threshold {
                println!("message to be posted: {},{}", m.timestamp, m.text);
                filtered_slack_messages.push(m);
            }
        }
        filtered_slack_messages
    }
}

struct FilterSlackMessageOptions {
    local_dt: DateTime<Local>,
    exclude_days: i64,
    exclude_hours: i64,
    exclude_minutes: i64,
}

impl FilterSlackMessageOptions {
    fn build(
        local_dt: DateTime<Local>,
        exclude_days: i64,
        exclude_hours: i64,
        exclude_minutes: i64,
    ) -> FilterSlackMessageOptions {
        FilterSlackMessageOptions {
            local_dt,
            exclude_days,
            exclude_hours,
            exclude_minutes,
        }
    }

    fn get_filter_threshold(&self) -> f64 {
        let local_dt = self.local_dt;
        let exclude_days = self.exclude_days;
        let exclude_hours = self.exclude_hours;
        let exclude_minutes = self.exclude_minutes;
        let threshold = (local_dt
            - Duration::days(exclude_days)
            - Duration::hours(exclude_hours)
            - Duration::minutes(exclude_minutes))
        .timestamp() as f64;
        threshold
    }
}
