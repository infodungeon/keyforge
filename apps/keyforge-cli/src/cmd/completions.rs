// apps/keyforge-cli/src/cmd/completions.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use clap::Args;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

#[derive(Args, Debug, Clone)]
pub struct CompletionsArgs {
    /// The shell to generate completions for (bash, zsh, fish, powershell, elvish)
    pub shell: Shell,
}

pub fn run(args: &CompletionsArgs) {
    let mut cmd = crate::Cli::command();
    let name = cmd.get_name().to_string();
    generate(args.shell, &mut cmd, name, &mut io::stdout());
}
