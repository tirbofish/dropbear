use clap::{Args, Subcommand};

#[derive(Subcommand, Debug)]
pub enum PluginCommand {
    /// Pack a plugin into a .eucplugin bundle.
    Pack(crate::commands::pack::PackArgs),
    /// Unpack a .eucplugin bundle into a directory.
    Unpack(crate::commands::unpack::UnpackArgs),
    /// Inspect the manifest of a .eucplugin bundle without extracting.
    Inspect(crate::commands::inspect::InspectArgs),
}

#[derive(Args, Debug)]
pub struct PluginArgs {
    #[command(subcommand)]
    pub command: PluginCommand,
}

impl PluginArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            PluginCommand::Pack(args) => crate::commands::pack::run(args),
            PluginCommand::Unpack(args) => crate::commands::unpack::run(args),
            PluginCommand::Inspect(args) => crate::commands::inspect::run(args),
        }
    }
}
