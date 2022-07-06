use anyhow::Result;
use dotenvy::dotenv;
use std::env;

use kakeibo_rs::ifttt::IFTTTAPIParams;
use kakeibo_rs::ifttt::IFTTTAPI;
use kakeibo_rs::slack::SlackAPI;
use kakeibo_rs::slack::SlackAPIParams;

#[cfg(not(tarpaulin_include))]
fn main() -> Result<()> {
    dotenv().ok();

    let slack_channel_id = env::var("SLACK_CHANNEL_ID").expect("$SLACK_CHANNEL_ID is not set");
    let slack_token = env::var("SLACK_TOKEN").expect("$SLACK_TOKEN is not set");
    let slack_messages =
        SlackAPI::new(SlackAPIParams::new(slack_channel_id, slack_token)).extract();
    slack_messages.iter().for_each(|m| {
        println!("{},{}", m.timestamp, m.text);
    });

    if !slack_messages.is_empty() {
        let ifttt_event_name = env::var("IFTTT_EVENT_NAME").expect("$IFTTT_EVENT_NAME is not set");
        let ifttt_webhook_token =
            env::var("IFTTT_WEBHOOK_TOKEN").expect("$IFTTT_WEBHOOK_TOKEN is not set");
        IFTTTAPI::new(IFTTTAPIParams::new(ifttt_event_name, ifttt_webhook_token))
            .kick(slack_messages);
    }
    Ok(())
}
