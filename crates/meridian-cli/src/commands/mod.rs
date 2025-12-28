pub mod import;
pub mod export;
pub mod query;
pub mod serve;
pub mod analyze;
pub mod db;

use std::path::PathBuf;

/// Common utilities for all commands
pub mod utils {
    use console::{style, Emoji};
    use indicatif::{ProgressBar, ProgressStyle};
    use std::time::Duration;

    pub static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
    pub static TRUCK: Emoji<'_, '_> = Emoji("🚚  ", "");
    pub static SPARKLE: Emoji<'_, '_> = Emoji("✨  ", "");
    pub static CLIP: Emoji<'_, '_> = Emoji("📎  ", "");

    pub fn create_progress_bar(len: u64, message: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .expect("Failed to create progress bar template")
                .progress_chars("#>-"),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    pub fn create_spinner(message: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Failed to create spinner template"),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }

    pub fn success(message: &str) {
        println!("{} {}", style("✓").green().bold(), style(message).green());
    }

    pub fn error(message: &str) {
        eprintln!("{} {}", style("✗").red().bold(), style(message).red());
    }

    pub fn info(message: &str) {
        println!("{} {}", style("ℹ").blue().bold(), message);
    }

    pub fn warning(message: &str) {
        println!("{} {}", style("⚠").yellow().bold(), style(message).yellow());
    }
}
