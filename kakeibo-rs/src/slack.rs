use anyhow::Result;
use chrono::{DateTime, Duration, Local};

const SLACK_BASE_URL: &str = "https://slack.com/api";
const SLACK_API_METHOD: &str = "conversations.history";
const EXCLUDE_DAYS: i64 = 0;
const EXCLUDE_HOURS: i64 = 0;
const EXCLUDE_MINUTES: i64 = 10;

#[derive(Debug, PartialEq, Clone)]
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
    pub fn new(slack_channel_id: String, slack_token: String) -> Self {
        Self {
            base_url: SLACK_BASE_URL.to_string(),
            method: SLACK_API_METHOD.to_string(),
            channel: slack_channel_id,
            token: slack_token,
        }
    }
}

pub trait SlackAPI {
    fn extract(&self) -> Result<Vec<SlackMessage>>;
}

pub struct SlackAPIClient {
    pub params: SlackAPIParams,
    threshold: f64,
}

impl SlackAPIClient {
    pub fn new(params: SlackAPIParams, local_dt: Option<DateTime<Local>>) -> Self {
        if let Some(local_dt) = local_dt {
            let fiter_options = FilterSlackMessageOptions::new(
                local_dt,
                EXCLUDE_DAYS,
                EXCLUDE_HOURS,
                EXCLUDE_MINUTES,
            );
            let threshold = fiter_options.get_threshold();
            Self { params, threshold }
        } else {
            let local_dt = Local::now();
            let fiter_options = FilterSlackMessageOptions::new(
                local_dt,
                EXCLUDE_DAYS,
                EXCLUDE_HOURS,
                EXCLUDE_MINUTES,
            );
            let threshold = fiter_options.get_threshold();
            Self { params, threshold }
        }
    }

    fn get_conversations_history(&self) -> Result<Vec<SlackMessage>> {
        let res = self.post();
        let res = match res {
            Ok(res) => self.json(res),
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "failed to get conversations history: {:?}",
                    e
                ));
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

    fn build_slack_messages(&self, res: &serde_json::Value) -> Result<Vec<SlackMessage>> {
        let messages = res["messages"].as_array().ok_or_else(|| {
            anyhow::anyhow!("failed to get messages from slack response: {:?}", res)
        })?;
        let slack_messages = messages
            .iter()
            .map(|message| {
                let timestamp = message["ts"].as_str().unwrap();
                let text = message["text"].as_str().unwrap();
                SlackMessage {
                    timestamp: timestamp.parse::<f64>().unwrap(),
                    text: text.to_string(),
                }
            })
            .collect();
        Ok(slack_messages)
    }

    fn filter(&self, slack_messages: Vec<SlackMessage>, threshold: f64) -> Vec<SlackMessage> {
        slack_messages
            .into_iter()
            .filter(|m| m.timestamp > threshold)
            .collect()
    }

    fn reverse<'a>(&self, slack_messages: &'a mut Vec<SlackMessage>) -> &'a mut Vec<SlackMessage> {
        slack_messages.reverse();
        slack_messages
    }
}

impl SlackAPI for SlackAPIClient {
    fn extract(&self) -> Result<Vec<SlackMessage>> {
        let slack_messages = self.get_conversations_history()?;
        let mut slack_messages = self.filter(slack_messages, self.threshold);
        let slack_messages = self.reverse(&mut slack_messages);
        Ok(slack_messages.clone())
    }
}

struct FilterSlackMessageOptions {
    local_dt: DateTime<Local>,
    exclude_days: i64,
    exclude_hours: i64,
    exclude_minutes: i64,
}

impl FilterSlackMessageOptions {
    fn new(
        local_dt: DateTime<Local>,
        exclude_days: i64,
        exclude_hours: i64,
        exclude_minutes: i64,
    ) -> Self {
        Self {
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
mod test {
    use super::*;

    const CHANNEL_ID: &str = "channel_id";
    const TOKEN: &str = "token";
    const EXCLUDE_DAYS: i64 = 1;
    const EXCLUDE_HOURS: i64 = 2;
    const EXCLUDE_MINUTES: i64 = 3;

    #[test]
    fn slack_api_params_new() {
        let params = SlackAPIParams::new(CHANNEL_ID.to_string(), TOKEN.to_string());
        assert_eq!(params.method, SLACK_API_METHOD);
        assert_eq!(params.channel, CHANNEL_ID);
        assert_eq!(params.token, TOKEN);
    }

    #[test]
    fn slack_api_new() {
        let params = SlackAPIParams::new(CHANNEL_ID.to_string(), TOKEN.to_string());
        let slack_client = SlackAPIClient::new(params, None);
        assert_eq!(slack_client.params.base_url, SLACK_BASE_URL);
        assert_eq!(slack_client.params.method, SLACK_API_METHOD);
        assert_eq!(slack_client.params.channel, CHANNEL_ID);
        assert_eq!(slack_client.params.token, TOKEN);
    }

    // FIXME: post の mock を作成する必要があるかも
    // #[test]
    // fn slack_api_post() {
    //     struct Fixture {
    //         name: &'static str,
    //         base_url: String,
    //         method: String,
    //         channel: String,
    //         token: String,
    //         expected: String,
    //     }
    //     let fixtures = vec![
    //         Fixture {
    //             name: "正常系",
    //             base_url: SLACK_BASE_URL.to_string(),
    //             method: SLACK_API_METHOD.to_string(),
    //             channel: CHANNEL_ID.to_string(),
    //             token: TOKEN.to_string(),
    //             expected: "invalid_auth".to_string(),
    //         },
    //         Fixture {
    //             name: "base_url が不正",
    //             base_url: "https://slack.com/api".to_string(),
    //             method: "method".to_string(),
    //             channel: "channel_id".to_string(),
    //             token: "token".to_string(),
    //             expected: "unknown_method".to_string(),
    //         },
    //         Fixture {
    //             name: "method が不正",
    //             base_url: "https://slack.com/api".to_string(),
    //             method: "method".to_string(),
    //             channel: "channel_id".to_string(),
    //             token: "token".to_string(),
    //             expected: "unknown_method".to_string(),
    //         },
    //         Fixture {
    //             name: "channel が不正",
    //             base_url: "https://slack.com/api".to_string(),
    //             method: "method".to_string(),
    //             channel: "channel_id".to_string(),
    //             token: "token".to_string(),
    //             expected: "unknown_method".to_string(),
    //         },
    //         Fixture {
    //             name: "token が不正",
    //             base_url: "https://slack.com/api".to_string(),
    //             method: "method".to_string(),
    //             channel: "channel_id".to_string(),
    //             token: "token".to_string(),
    //             expected: "unknown_method".to_string(),
    //         },
    //     ];
    //     for fixture in fixtures.iter() {
    //         let slack_api = SlackAPI::new(SlackAPIParams {
    //             base_url: fixture.base_url.clone(),
    //             method: fixture.method.clone(),
    //             channel: fixture.channel.clone(),
    //             token: fixture.token.clone(),
    //         });
    //         let res = slack_api.post();
    //         let res = match res {
    //             Ok(res) => slack_api.json(res),
    //             Err(e) => panic!("{} {}", fixture.name, e),
    //         };
    //         assert!(!res["ok"].as_bool().unwrap(), "{}", fixture.name);
    //         assert_eq!(res["error"], fixture.expected.clone(), "{}", fixture.name);
    //     }
    // }

    #[test]
    fn test_build_slack_messages() {
        let slack_client = SlackAPIClient::new(
            SlackAPIParams {
                base_url: SLACK_BASE_URL.to_string(),
                method: SLACK_API_METHOD.to_string(),
                channel: CHANNEL_ID.to_string(),
                token: TOKEN.to_string(),
            },
            None,
        );
        let res: serde_json::Value = serde_json::from_str(
            r#"{
            "ok": true,
            "messages": [
                {
                    "text": "text1",
                    "ts": "1589788800.000001"
                },
                {
                    "text": "text2",
                    "ts": "1589788800.000002"
                }
            ]
        }"#,
        )
        .unwrap();
        let actual = slack_client.build_slack_messages(&res).unwrap();
        let expected = vec![
            SlackMessage {
                text: "text1".to_string(),
                timestamp: 1589788800.000001,
            },
            SlackMessage {
                text: "text2".to_string(),
                timestamp: 1589788800.000002,
            },
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn slack_api_filter() {
        let slack_client = SlackAPIClient::new(
            SlackAPIParams::new(CHANNEL_ID.to_string(), TOKEN.to_string()),
            None,
        );

        let slack_messages = vec![
            SlackMessage {
                timestamp: 1.0,
                text: "test1".to_string(),
            },
            SlackMessage {
                timestamp: 2.0,
                text: "test2".to_string(),
            },
            SlackMessage {
                timestamp: 3.0,
                text: "test3".to_string(),
            },
        ];
        let expected = vec![
            SlackMessage {
                timestamp: 2.0,
                text: "test2".to_string(),
            },
            SlackMessage {
                timestamp: 3.0,
                text: "test3".to_string(),
            },
        ];
        let threshold = 1.0;
        let filtered_slack_messages = slack_client.filter(slack_messages, threshold);
        assert_eq!(&expected, &filtered_slack_messages);
    }

    #[test]
    fn slack_api_reverse() {
        let slack_client = SlackAPIClient::new(
            SlackAPIParams::new(CHANNEL_ID.to_string(), TOKEN.to_string()),
            None,
        );

        let mut slack_messages = vec![
            SlackMessage {
                timestamp: 1.0,
                text: "test1".to_string(),
            },
            SlackMessage {
                timestamp: 2.0,
                text: "test2".to_string(),
            },
            SlackMessage {
                timestamp: 3.0,
                text: "test3".to_string(),
            },
        ];
        let mut expected = vec![
            SlackMessage {
                timestamp: 3.0,
                text: "test3".to_string(),
            },
            SlackMessage {
                timestamp: 2.0,
                text: "test2".to_string(),
            },
            SlackMessage {
                timestamp: 1.0,
                text: "test1".to_string(),
            },
        ];
        let reversed_slack_messages = slack_client.reverse(&mut slack_messages);
        assert_eq!(&mut expected, reversed_slack_messages);
    }

    #[test]
    fn filter_slack_messages_option_new() {
        let local_dt = Local::now();
        let fiter_options =
            FilterSlackMessageOptions::new(local_dt, EXCLUDE_DAYS, EXCLUDE_HOURS, EXCLUDE_MINUTES);
        assert_eq!(fiter_options.local_dt, local_dt);
        assert_eq!(fiter_options.exclude_days, EXCLUDE_DAYS);
        assert_eq!(fiter_options.exclude_hours, EXCLUDE_HOURS);
        assert_eq!(fiter_options.exclude_minutes, EXCLUDE_MINUTES);
    }

    #[test]
    fn filter_slack_messages_option_get_filter_threshold() {
        let local_dt = Local::now();
        let fiter_options = FilterSlackMessageOptions::new(local_dt, 1, 2, 3);
        let threshold = fiter_options.get_threshold();
        let expected: f64 = (local_dt
            - Duration::days(EXCLUDE_DAYS)
            - Duration::hours(EXCLUDE_HOURS)
            - Duration::minutes(EXCLUDE_MINUTES))
        .timestamp() as f64;
        assert_eq!(threshold, expected);
    }
}
