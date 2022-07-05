use anyhow::Result;
use chrono::{DateTime, Duration, Local};
use std::env;

const SLACK_BASE_URL: &str = "https://slack.com/api";
const SLACK_API_METHOD: &str = "conversations.history";
const EXCLUDE_DAYS: i64 = 0;
const EXCLUDE_HOURS: i64 = 0;
const EXCLUDE_MINUTES: i64 = 10;

#[derive(Debug, PartialEq)]
pub struct SlackMessage {
    pub timestamp: f64,
    pub text: String,
}

pub struct SlackAPIParams {
    base_url: String,
    method: String,
    channel: String,
    token: String,
}

impl SlackAPIParams {
    pub fn build() -> SlackAPIParams {
        SlackAPIParams {
            base_url: SLACK_BASE_URL.to_string(),
            method: SLACK_API_METHOD.to_string(),
            channel: env::var("SLACK_CHANNEL_ID").expect("$SLACK_CHANNEL_ID is not set"),
            token: env::var("SLACK_TOKEN").expect("$SLACK_TOKEN is not set"),
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
        let threshold = fiter_options.get_threshold();
        self.filter(slack_messages, threshold)
    }

    fn get_conversations_history(&self) -> Vec<SlackMessage> {
        let res = self.post();
        let res = match res {
            Ok(res) => self.json(res),
            Err(e) => {
                println!("{}", e);
                return vec![];
            }
        };
        self.build_slack_messages(&res)
    }

    fn post(&self) -> Result<reqwest::blocking::Response, reqwest::Error> {
        let slack_url = format!(
            "{}/{}?channel={}",
            self.params.base_url, self.params.method, self.params.channel
        );
        let slack_header_auth = format!("Bearer {}", self.params.token);

        let slack_client = reqwest::blocking::Client::new();
        slack_client
            .post(slack_url)
            .header("Authorization", slack_header_auth)
            .send()
    }

    fn json(&self, res: reqwest::blocking::Response) -> serde_json::Value {
        res.json().expect("failed to deserialize json")
    }

    fn build_slack_messages(&self, res: &serde_json::Value) -> Vec<SlackMessage> {
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

    fn get_threshold(&self) -> f64 {
        let local_dt = self.local_dt;
        let exclude_days = self.exclude_days;
        let exclude_hours = self.exclude_hours;
        let exclude_minutes = self.exclude_minutes;
        (local_dt
            - Duration::days(exclude_days)
            - Duration::hours(exclude_hours)
            - Duration::minutes(exclude_minutes))
        .timestamp() as f64
    }
}

#[cfg(test)]
mod test_slack_api_params {
    use super::*;

    const CHANNEL_ID: &str = "channel_id";
    const TOKEN: &str = "token";

    fn set_up_dummy_env() {
        env::set_var("SLACK_CHANNEL_ID", CHANNEL_ID);
        env::set_var("SLACK_TOKEN", TOKEN);
    }

    #[test]
    fn build() {
        set_up_dummy_env();
        let params = SlackAPIParams::build();
        assert_eq!(params.method, SLACK_API_METHOD);
        assert_eq!(params.channel, CHANNEL_ID);
        assert_eq!(params.token, TOKEN);
    }
}

#[cfg(test)]
mod test_slack_api {
    use super::*;

    const CHANNEL_ID: &str = "channel_id";
    const TOKEN: &str = "token";

    fn set_up_dummy_env() {
        env::set_var("SLACK_CHANNEL_ID", CHANNEL_ID);
        env::set_var("SLACK_TOKEN", TOKEN);
    }

    #[test]
    fn post() {
        struct Fixture {
            name: &'static str,
            base_url: String,
            method: String,
            channel: String,
            token: String,
            expected: String,
        }
        let fixtures = vec![
            Fixture {
                name: "正常系",
                base_url: SLACK_BASE_URL.to_string(),
                method: SLACK_API_METHOD.to_string(),
                channel: env::var("SLACK_CHANNEL_ID").unwrap(),
                token: env::var("SLACK_TOKEN").unwrap(),
                expected: "invalid_auth".to_string(),
            },
            Fixture {
                name: "base_url が不正",
                base_url: "https://slack.com/api".to_string(),
                method: "method".to_string(),
                channel: "channel_id".to_string(),
                token: "token".to_string(),
                expected: "unknown_method".to_string(),
            },
            Fixture {
                name: "method が不正",
                base_url: "https://slack.com/api".to_string(),
                method: "method".to_string(),
                channel: "channel_id".to_string(),
                token: "token".to_string(),
                expected: "unknown_method".to_string(),
            },
            Fixture {
                name: "channel が不正",
                base_url: "https://slack.com/api".to_string(),
                method: "method".to_string(),
                channel: "channel_id".to_string(),
                token: "token".to_string(),
                expected: "unknown_method".to_string(),
            },
            Fixture {
                name: "token が不正",
                base_url: "https://slack.com/api".to_string(),
                method: "method".to_string(),
                channel: "channel_id".to_string(),
                token: "token".to_string(),
                expected: "unknown_method".to_string(),
            },
        ];
        for fixture in fixtures.iter() {
            let slack_api = SlackAPI {
                params: SlackAPIParams {
                    base_url: fixture.base_url.clone(),
                    method: fixture.method.clone(),
                    channel: fixture.channel.clone(),
                    token: fixture.token.clone(),
                },
            };
            let res = slack_api.post();
            let res = match res {
                Ok(res) => slack_api.json(res),
                Err(e) => panic!("{} {}", fixture.name, e),
            };
            assert!(!res["ok"].as_bool().unwrap(), "{}", fixture.name);
            assert_eq!(res["error"], fixture.expected.clone(), "{}", fixture.name);
        }
    }

    #[test]
    fn filter() {
        set_up_dummy_env();

        let slack_api = SlackAPI {
            params: SlackAPIParams::build(),
        };

        let slack_messages = vec![SlackMessage {
            timestamp: 1578472400.0,
            text: "test".to_string(),
        }];
        let expected = vec![SlackMessage {
            timestamp: 1578472400.0,
            text: "test".to_string(),
        }];
        let threshold = 0.0;
        let filtered_slack_messages = slack_api.filter(slack_messages, threshold);
        assert_eq!(&expected, &filtered_slack_messages);
    }
}

#[cfg(test)]
mod test_filter_slack_messages_option {
    use super::*;

    const EXCLUDE_DAYS: i64 = 1;
    const EXCLUDE_HOURS: i64 = 2;
    const EXCLUDE_MINUTES: i64 = 3;

    #[test]
    fn build() {
        let local_dt = Local::now();
        let fiter_options = FilterSlackMessageOptions::build(
            local_dt,
            EXCLUDE_DAYS,
            EXCLUDE_HOURS,
            EXCLUDE_MINUTES,
        );
        assert_eq!(fiter_options.local_dt, local_dt);
        assert_eq!(fiter_options.exclude_days, EXCLUDE_DAYS);
        assert_eq!(fiter_options.exclude_hours, EXCLUDE_HOURS);
        assert_eq!(fiter_options.exclude_minutes, EXCLUDE_MINUTES);
    }

    #[test]
    fn get_filter_threshold() {
        let local_dt = Local::now();
        let fiter_options = FilterSlackMessageOptions::build(local_dt, 1, 2, 3);
        let threshold = fiter_options.get_threshold();
        let expected: f64 = (local_dt
            - Duration::days(EXCLUDE_DAYS)
            - Duration::hours(EXCLUDE_HOURS)
            - Duration::minutes(EXCLUDE_MINUTES))
        .timestamp() as f64;
        assert_eq!(threshold, expected);
    }
}
