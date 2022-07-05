use anyhow::Result;

use kakeibo_rs::ifttt::IFTTTAPIParams;
use kakeibo_rs::ifttt::IFTTTAPI;
use kakeibo_rs::slack::SlackAPI;
use kakeibo_rs::slack::SlackAPIParams;

fn main() -> Result<()> {
    let slack_messages = SlackAPI {
        params: SlackAPIParams::build(),
    }
    .extract();
    IFTTTAPI {
        params: IFTTTAPIParams::build(),
    }
    .kick(slack_messages);
    Ok(())
}
