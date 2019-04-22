use crate::application::core_types::logger::LogLevel;
use crate::application::core_types::notify::NotifyError;
use serde_derive::Deserialize;
use slack_hook::{PayloadBuilder, Slack};

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifyMethod {
    Slack {
        url: String,
        channel: String,
        mention: Option<String>,
        level: LogLevel,
    },
}

pub fn send_to_slack(
    url: &str,
    channel: &str,
    mention: Option<&String>,
    username: &str,
    level: &LogLevel,
    label: &str,
    message: &str,
) -> Result<(), NotifyError> {
    let slack_result = Slack::new(url as &str);
    let mention_str: &str = mention
        .as_ref()
        .map(|mention| mention as &str)
        .unwrap_or("");

    let payload_result = PayloadBuilder::new()
        .channel(channel as &str)
        .username(format!("raven - {}", username))
        .link_names(true)
        .text(format!(
            "{} *`[{}] {}`*\n ```{}```",
            mention_str,
            level.to_str(),
            label,
            message
        ))
        .build();

    let mut err_msgs: Vec<String> = vec![];
    match (&slack_result, &payload_result) {
        (Ok(slack), Ok(payload)) => {
            let send_result = slack.send(payload);
            if let Err(e) = send_result {
                err_msgs.push(format!("failed to send to slack: {:?}", e));
            }
        }
        (Err(e), _) => err_msgs.push(format!("failed to build slack client: {:?}", e)),

        (_, Err(e)) => err_msgs.push(format!("failed to build payload: {:?}", e)),
    }

    if err_msgs.is_empty() {
        Ok(())
    } else {
        Err(NotifyError(format!(
            "failed to notify to slack: {}",
            err_msgs.join(", ")
        )))
    }
}
