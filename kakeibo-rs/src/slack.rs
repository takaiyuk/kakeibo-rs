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
    client: reqwest::blocking::Client,
    slack_url: String,
    threshold: f64,
}

impl SlackAPIClient {
    pub fn new(params: SlackAPIParams) -> Self {
        let client = reqwest::blocking::Client::new();
        let local_dt = Local::now();
        let fiter_options =
            FilterSlackMessageOptions::new(local_dt, EXCLUDE_DAYS, EXCLUDE_HOURS, EXCLUDE_MINUTES);
        let slack_url = Self::build_slack_url(&params);
        let threshold = fiter_options.get_threshold();
        Self { params, client, slack_url, threshold }
    }

    fn build_slack_url(params: &SlackAPIParams) -> String {
        format!(
            "{}/{}?channel={}",
            params.base_url, params.method, params.channel
        )
    }

    fn get_conversations_history(&self, slack_url: String) -> Result<Vec<SlackMessage>> {
        let res = self.post(slack_url);
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

    fn post(&self, slack_url: String) -> Result<reqwest::blocking::Response, reqwest::Error> {
        let slack_header_auth = format!("Bearer {}", self.params.token);

        self.client
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
        let slack_messages = self.get_conversations_history(self.slack_url.clone())?;
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
    const PATH: &str = "/test";
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
        let slack_client = SlackAPIClient::new(params);
        assert_eq!(slack_client.params.base_url, SLACK_BASE_URL);
        assert_eq!(slack_client.params.method, SLACK_API_METHOD);
        assert_eq!(slack_client.params.channel, CHANNEL_ID);
        assert_eq!(slack_client.params.token, TOKEN);
    }

    #[test]
    fn slack_api_extract() {
        // Mock server
        let mut server = mockito::Server::new();
        let mock_url = format!("{}{}", server.url(), PATH);
        server
            .mock("POST", PATH)
            .with_status(200)
            .with_header("Authorization", format!("Bearer {}", TOKEN.clone()).as_str())
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "ok": true,
                "messages": [
                    {
                        "text": "text1",
                        "ts": "1589788800.000001"
                    }
                ]
            }"#)
            .create();

        let mut slack_client = SlackAPIClient::new(SlackAPIParams {
            base_url: SLACK_BASE_URL.to_string(),
            method: SLACK_API_METHOD.to_string(),
            channel: CHANNEL_ID.to_string(),
            token: TOKEN.to_string(),
        });
        slack_client.slack_url = mock_url;
        let actual = slack_client.extract().unwrap();
        let expected = vec![];
        assert_eq!(actual, expected);
    }

    #[test]
    fn slack_api_build_slack_url() {
        let params = SlackAPIParams::new(CHANNEL_ID.to_string(), TOKEN.to_string());
        let slack_url = SlackAPIClient::build_slack_url(&params);
        assert_eq!(
            slack_url,
            format!(
                "{}/{}?channel={}",
                SLACK_BASE_URL, SLACK_API_METHOD, CHANNEL_ID
            )
        );
    }

    #[test]
    fn slack_api_get_conversations_history() {
        // Mock server
        let mut server = mockito::Server::new();
        let mock_url = format!("{}{}", server.url(), PATH);
        server
            .mock("POST", PATH)
            .with_status(200)
            .with_header("Authorization", format!("Bearer {}", TOKEN.clone()).as_str())
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "ok": true,
                "messages": [
                    {
                        "text": "text1",
                        "ts": "1589788800.000001"
                    }
                ]
            }"#)
            .create();

        let slack_client = SlackAPIClient::new(SlackAPIParams {
            base_url: SLACK_BASE_URL.to_string(),
            method: SLACK_API_METHOD.to_string(),
            channel: CHANNEL_ID.to_string(),
            token: TOKEN.to_string(),
        });
        let actual = slack_client.get_conversations_history(mock_url).unwrap();
        let expected = vec![
            SlackMessage {
                text: "text1".to_string(),
                timestamp: 1589788800.000001,
            },
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn slack_api_post() {
        // Mock server
        let mut server = mockito::Server::new();
        let mock_url = format!("{}{}", server.url(), PATH);
        server
            .mock("POST", PATH)
            .with_status(200)
            .with_header("Authorization", format!("Bearer {}", TOKEN.clone()).as_str())
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "ok": true,
                "messages": []
            }"#)
            .create();

        let slack_client = SlackAPIClient::new(SlackAPIParams {
            base_url: SLACK_BASE_URL.to_string(),
            method: SLACK_API_METHOD.to_string(),
            channel: CHANNEL_ID.to_string(),
            token: TOKEN.to_string(),
        });
        let res = slack_client.post(mock_url);
        let res = match res {
            Ok(res) => slack_client.json(res),
            Err(e) => panic!("failed to post: {:?}", e),
        };
        assert!(res["ok"].as_bool().unwrap());
    }

    #[test]
    fn test_build_slack_messages() {
        let slack_client = SlackAPIClient::new(SlackAPIParams {
            base_url: SLACK_BASE_URL.to_string(),
            method: SLACK_API_METHOD.to_string(),
            channel: CHANNEL_ID.to_string(),
            token: TOKEN.to_string(),
        });
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
        let slack_client = SlackAPIClient::new(SlackAPIParams::new(
            CHANNEL_ID.to_string(),
            TOKEN.to_string(),
        ));

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
        let slack_client = SlackAPIClient::new(SlackAPIParams::new(
            CHANNEL_ID.to_string(),
            TOKEN.to_string(),
        ));

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
