use super::Action;
use crate::message::MessageConfig;
use crate::message::MessageParams;
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug)]
pub struct DesktopNotificationAction<'a> {
    message_config: &'a MessageConfig,
}

impl<'a> DesktopNotificationAction<'a> {
    pub fn new(message_config: &'a MessageConfig) -> Self {
        Self { message_config }
    }
}

#[async_trait]
impl Action<'_> for DesktopNotificationAction<'_> {
    fn name(&self) -> &'static str {
        "desktop_notification"
    }

    async fn run(&self, params: Option<&MessageParams<'_, '_>>) -> Result<()> {
        use notify_rust::Notification;
        Notification::new()
            .summary(&self.message_config.subject(params)?)
            .body(&self.message_config.body(params)?)
            .show()?;
        Ok(())
    }
}
