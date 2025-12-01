use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    /// Path to the `.tsk` file
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let xml = std::fs::read_to_string(&args.path)?;
    let task = xcsoar_tasks::parse(&xml)?;
    println!("{task:#?}");
    Ok(())
}
