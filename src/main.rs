use dotenv::dotenv;
use kakeibo_rs::ifttt::kick_ifttt_webhook;
use kakeibo_rs::slack::extract_slack_messages;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let slack_messages = extract_slack_messages();
    kick_ifttt_webhook(slack_messages);
    Ok(())
}
