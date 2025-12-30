use clap::Args;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

#[derive(Args, Debug, Clone)]
pub struct CompletionsArgs {
    /// The shell to generate completions for (bash, zsh, fish, powershell, elvish)
    pub shell: Shell,
}

pub fn run(args: CompletionsArgs) {
    let mut cmd = crate::Cli::command();
    let name = cmd.get_name().to_string();
    generate(args.shell, &mut cmd, name, &mut io::stdout());
}
