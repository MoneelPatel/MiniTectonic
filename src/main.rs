use mini_tectonic_rs::cli;

fn main() -> mini_tectonic_rs::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Run the CLI
    cli::run()
} 