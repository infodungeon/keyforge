// apps/keyforge-cli/src/cmd/benchmark.rs

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
use keyforge_compute::Runtime;
use keyforge_model::KeyCode;
use std::time::Instant;

#[derive(Args, Debug, Clone)]
pub struct BenchmarkArgs {
    #[command(flatten)]
    pub config: crate::cli_args::config::ConfigArgs,
    #[arg(long, default_value_t = 100_000)]
    pub iterations: usize,
    #[command(flatten)]
    pub shared: crate::cmd::shared::SharedArgs,
}

pub fn run(args: BenchmarkArgs, runtime: Runtime) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("📊 Benchmarking Engine ({} iterations)...", args.iterations);

    // Create a dummy layout for benchmarking
    // We need a layout that matches the key count of the engine
    let key_count = runtime.engine.key_count();
    let layout = keyforge_model::Layout::new_unchecked((0..key_count).map(|i| KeyCode(i as u16)).collect());

    let start = Instant::now();
    let mut score = 0.0;

    for _ in 0..args.iterations {
        score += runtime.score(&layout)?;
    }

    let duration = start.elapsed();
    let kops = (args.iterations as f64 / duration.as_secs_f64()) / 1000.0;

    eprintln!("Finished in {:.2?}", duration);
    eprintln!("Throughput: {:.2} kOPS", kops);
    eprintln!("CheckSum: {:.2}", score); // Prevent optimization

    Ok(())
}
