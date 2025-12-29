use clap::Args;
use keyforge_infra::load_keycode_registry;
use keyforge_protocol::keycodes::KeycodeRegistry;
use keyforge_model::KeyCode;
use std::fmt::Write;
use std::path::Path;

#[derive(Args, Debug, Clone)]
pub struct FmtArgs {
    pub layout: String,
    #[arg(short, long, default_value_t = 10)]
    pub width: usize,
}

pub fn run(args: FmtArgs, root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let kc_path = root.join("keycodes.json");
    let registry = if kc_path.exists() {
        load_keycode_registry(&kc_path).unwrap_or_else(|_| KeycodeRegistry::new_with_defaults())
    } else {
        KeycodeRegistry::new_with_defaults()
    };

    let layout = keyforge_adapter::conversion::parse_layout_string(&args.layout, 200, &registry)
        .map_err(|e| format!("Failed to parse layout: {}", e))?;

    let valid_codes: Vec<KeyCode> = layout.keys.into_iter().filter(|&c| c != KeyCode(0)).collect();

    if valid_codes.is_empty() {
        println!();
        return Ok(());
    }

    let mut output = String::with_capacity(valid_codes.len() * 7);
    for (i, code) in valid_codes.iter().enumerate() {
        let label = registry.get_label(*code);
        let _ = write!(output, "{:<6}", label);
        if (i + 1) % args.width == 0 {
            output.push('\n');
        } else {
            output.push(' ');
        }
    }

    println!("{}", output.trim());
    Ok(())
}
