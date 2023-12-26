use anyhow::Result;
use dotenvy::dotenv;
use std::env;

use crate::ifttt::IFTTTAPIParams;
use crate::ifttt::{IFTTTAPIClient, IFTTTAPI};
use crate::slack::SlackAPIParams;
use crate::slack::{SlackAPI, SlackAPIClient};

#[cfg(not(tarpaulin_include))]
pub fn run_kakeibo() -> Result<()> {
    dotenv().ok();

    let slack_channel_id = env::var("SLACK_CHANNEL_ID").expect("$SLACK_CHANNEL_ID is not set");
    let slack_token = env::var("SLACK_TOKEN").expect("$SLACK_TOKEN is not set");
    let slack_client = SlackAPIClient::new(SlackAPIParams::new(slack_channel_id, slack_token));
    let slack_messages = slack_client.extract()?;
    slack_messages.iter().for_each(|m| {
        println!("{},{}", m.timestamp, m.text);
    });

    if !slack_messages.is_empty() {
        let ifttt_event_name = env::var("IFTTT_EVENT_NAME").expect("$IFTTT_EVENT_NAME is not set");
        let ifttt_webhook_token =
            env::var("IFTTT_WEBHOOK_TOKEN").expect("$IFTTT_WEBHOOK_TOKEN is not set");
        let ifttt_api_params = IFTTTAPIParams::new(ifttt_event_name, ifttt_webhook_token);
        let ifttt_client = IFTTTAPIClient::new(ifttt_api_params);
        ifttt_client.kick(slack_messages);
    }

    Ok(())
}
