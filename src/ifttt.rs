use anyhow::Result;
use std::collections::HashMap;

use crate::slack::SlackMessage;

const IFTTT_BASE_URL: &str = "https://maker.ifttt.com/trigger";

pub struct IFTTTAPIParams {
    event_name: String,
    token: String,
}

impl IFTTTAPIParams {
    pub fn new(ifttt_event_name: String, ifttt_webhook_token: String) -> Self {
        Self {
            event_name: ifttt_event_name,
            token: ifttt_webhook_token,
        }
    }
}

pub struct IFTTTAPI {
    pub params: IFTTTAPIParams,
    client: reqwest::blocking::Client,
}

impl IFTTTAPI {
    pub fn new(params: IFTTTAPIParams) -> Self {
        let client = reqwest::blocking::Client::new();
        Self { params, client }
    }

    pub fn kick(&self, slack_messages: Vec<SlackMessage>) {
        let ifttt_url = self.build_ifttt_url();
        for m in slack_messages {
            let payload = self.build_payload(&m);
            let res = self.post_ifttt_webhook(&ifttt_url, payload);
            match res {
                Ok(_) => println!("message posted: {},{}", m.timestamp, m.text),
                Err(e) => {
                    println!("Error sending IFTTT webhook: StatusCode: {:?}", e.status());
                }
            }
        }
    }

    fn build_ifttt_url(&self) -> String {
        format!(
            "{}/{}/with/key/{}",
            IFTTT_BASE_URL, self.params.event_name, self.params.token
        )
    }

    fn build_payload(&self, m: &SlackMessage) -> String {
        let mut payload = HashMap::new();
        payload.insert("value1", m.timestamp.to_string());
        payload.insert("value2", m.text.to_string());
        serde_json::to_string(&payload).unwrap()
    }

    fn post_ifttt_webhook(
        &self,
        ifttt_url: &String,
        payload: String,
    ) -> Result<reqwest::blocking::Response, reqwest::Error> {
        self.client
            .post(ifttt_url)
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EVENT_NAME: &str = "channel_id";
    const TOKEN: &str = "token";

    #[test]
    fn ifttt_api_params_new() {
        let params = IFTTTAPIParams::new(EVENT_NAME.to_string(), TOKEN.to_string());
        assert_eq!(params.event_name, EVENT_NAME);
        assert_eq!(params.token, TOKEN);
    }

    #[test]
    fn ifttt_api_new() {
        let params = IFTTTAPIParams::new(EVENT_NAME.to_string(), TOKEN.to_string());
        let api = IFTTTAPI::new(params);
        assert_eq!(api.params.event_name, EVENT_NAME);
        assert_eq!(api.params.token, TOKEN);
    }

    #[test]
    fn ifttt_api_build_ifttt_url() {
        let params = IFTTTAPIParams::new(EVENT_NAME.to_string(), TOKEN.to_string());
        let api = IFTTTAPI::new(params);
        let url = api.build_ifttt_url();
        assert_eq!(
            url,
            format!("{}/{}/with/key/{}", IFTTT_BASE_URL, EVENT_NAME, TOKEN)
        );
    }

    #[test]
    fn ifttt_api_build_payload() {
        let m = SlackMessage {
            timestamp: 12345.0,
            text: "test".to_string(),
        };
        let expected = r#"{"value1":"12345","value2":"test"}"#.to_string();
        let params = IFTTTAPIParams::new(EVENT_NAME.to_string(), TOKEN.to_string());
        let api = IFTTTAPI::new(params);
        let actual = api.build_payload(&m);

        let actual_des: HashMap<String, String> = serde_json::from_str(&actual).unwrap();
        let expected_des: HashMap<String, String> = serde_json::from_str(&expected).unwrap();
        assert_eq!(actual_des, expected_des);
    }

    // FIXME: post の mock を作成する必要があるかも
    // #[test]
    // fn ifttt_api_post_ifttt_webhook() {
    //     let m = SlackMessage {
    //         timestamp: 12345.0,
    //         text: "test".to_string(),
    //     };
    //     let params = IFTTTAPIParams::new(EVENT_NAME.to_string(), TOKEN.to_string());
    //     let api = IFTTTAPI::new(params);
    //     let url = api.build_ifttt_url();
    //     let payload = api.build_payload(&m);
    //     let actual = api.post_ifttt_webhook(&url, payload);
    //     let expected = reqwest::StatusCode::UNAUTHORIZED;
    //     assert_eq!(expected, actual.unwrap().status());
    // }
}
