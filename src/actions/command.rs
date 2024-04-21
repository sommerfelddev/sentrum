use std::collections::HashMap;
use std::process::Command;

use super::Action;
use crate::message::MessageConfig;
use crate::message::MessageParams;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CommandConfig {
    cmd: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    clear_parent_env: bool,
    #[serde(default)]
    envs: HashMap<String, String>,
    working_dir: Option<String>,
}

pub struct CommandAction<'a> {
    message_config: &'a MessageConfig,
    cmd_config: &'a CommandConfig,
}

impl<'a> CommandAction<'a> {
    pub fn new(message_config: &'a MessageConfig, cmd_config: &'a CommandConfig) -> Result<Self> {
        Ok(Self {
            message_config,
            cmd_config,
        })
    }
}

#[async_trait]
impl Action<'_> for CommandAction<'_> {
    async fn run(&self, params: Option<&MessageParams<'_, '_>>) -> Result<()> {
        let mut cmd = Command::new(&self.cmd_config.cmd);
        for arg in self.cmd_config.args.iter() {
            cmd.arg(if let Some(p) = params {
                self.message_config.replace_template_params(arg, p)?
            } else {
                arg.clone()
            });
        }

        if self.cmd_config.clear_parent_env {
            cmd.env_clear();
        }
        cmd.envs(&self.cmd_config.envs);

        if let Some(working_dir) = &self.cmd_config.working_dir {
            cmd.current_dir(working_dir);
        }

        cmd.status()?;
        Ok(())
    }
}
