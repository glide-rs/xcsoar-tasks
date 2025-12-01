mod geojson;

use clap::Parser;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    /// Path to the .tsk file
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let xml = std::fs::read_to_string(&args.path)?;
    let task = xcsoar_tasks::parse(&xml)?;
    let geojson = geojson::task_to_geojson(&task);

    let template = include_str!("template.html.j2");
    let html = minijinja::render!(template, geojson);

    let mut temp_file = tempfile::Builder::new()
        .prefix("task_")
        .suffix(".html")
        .tempfile()?;

    temp_file.write_all(html.as_bytes())?;

    let (_, path) = temp_file.keep()?;
    let file_url = format!("file://{}", path.display());

    println!("Opening: {}", path.display());
    webbrowser::open(&file_url)?;

    Ok(())
}
