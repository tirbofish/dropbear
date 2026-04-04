use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Operations on plugin bundles (.eucplugin).
    Plugin(crate::commands::plugin::PluginArgs),
}

#[derive(Parser, Debug)]
#[command(version, about = "CLI tooling for eucalyptus")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn match_command(&self) -> anyhow::Result<()> {
        match &self.command {
            Command::Plugin(args) => args.run(),
        }
    }
}