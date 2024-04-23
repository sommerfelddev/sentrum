use super::Action;
use crate::message::MessageConfig;
use crate::message::MessageParams;
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug)]
pub struct TerminalPrintAction<'a> {
    message_config: &'a MessageConfig,
}

impl<'a> TerminalPrintAction<'a> {
    pub fn new(message_config: &'a MessageConfig) -> Self {
        Self { message_config }
    }
}

#[async_trait]
impl Action<'_> for TerminalPrintAction<'_> {
    fn name(&self) -> &'static str { "terminal_print" }

    async fn run(&self, params: Option<&MessageParams<'_, '_>>) -> Result<()> {
        println!(
            "{}\n{}\n",
            self.message_config.subject(params)?,
            self.message_config.body(params)?
        );
        Ok(())
    }
}
