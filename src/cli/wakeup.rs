#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use anyhow::Result;
use colored::Colorize;
use crate::config::Config;
use crate::layers::{Layer0, Layer1, LayerStack};

/// Handle the wake-up command
pub async fn handle_wakeup(config: &Config, wing: Option<String>) -> Result<()> {
    println!("{}", "╔═══════════════════════════════════════════╗".cyan());
    println!("{}", "║           Wake-Up Context                 ║".cyan());
    println!("{}", "╚═══════════════════════════════════════════╝".cyan());
    println!();

    // Initialize layer stack
    let mut layer_stack = LayerStack::new(config)?;

    // Layer 0: Identity
    println!("{}", "═ Layer 0: Identity ════════════════════════".blue().bold());
    println!();

    let identity = layer_stack.layer0.render();
    if identity.is_empty() {
        println!("{}", "⚠️  No identity configured.".yellow());
        println!();
        println!("Create {} to define your AI's identity.",
            config.layers.identity_file.display().to_string().cyan()
        );
    } else {
        println!("{}", identity);
    }

    println!();

    // Layer 1: Essential Story
    println!("{}", "═ Layer 1: Essential Story ═════════════════".blue().bold());
    println!();

    let essential = if let Some(ref w) = wing {
        layer_stack.layer1.render_for_wing(w, &mut layer_stack.store)?
    } else {
        layer_stack.layer1.render(&mut layer_stack.store)?
    };

    if essential.is_empty() {
        println!("{}", "⚠️  No essential story available.".yellow());
    } else {
        println!("{}", essential);
    }

    println!();

    // Token estimate
    let total_tokens = layer_stack.estimate_tokens();
    println!("{}", format!("📊 Estimated tokens: ~{}", total_tokens).dimmed());
    println!();

    if wing.is_some() {
        println!("💡 Tip: Use {} for general wake-up context.",
            "mempalace wake-up".cyan()
        );
    } else {
        println!("💡 Tip: Use {} for wing-specific context.",
            "mempalace wake-up --wing <wing>".cyan()
        );
    }

    Ok(())
}
