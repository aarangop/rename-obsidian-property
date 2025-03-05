mod utils;

use std::error::Error ;

use clap::Parser;
use utils::{load_obsidian_notes, ObsidianNoteProcessor};

/// This is a small utility program to rename Obsidian properties. 

// Setup cli
#[derive(Parser)]
#[command(version="0.1", about, long_about="Utility to rename Obsidian properties")]
struct Cli {
    property: String,
    new_name: String,
    
    #[arg(short, long, value_name="FILE_PATTERN")]
    pattern: Option<String>,

    /// Run in dry-run mode (don't actually modify files)
    #[arg(short, long, default_value_t = false)]
    dry_run: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Init cli.
    let cli = Cli::parse();

    // Extract cli args.
    let property = cli.property;
    let new_name = cli.new_name;
    let pattern = match cli.pattern {
        Some(glob) => glob,
        None => "./**/*.md".to_string()
    };

    println!("Renaming property {:?} to {:?} in notes matching {:?}", property, new_name, pattern);

    // Load obsidian notes
    let obsidian_notes = load_obsidian_notes(&pattern)?;

    // Create a note processor for batch processing
    let mut processor = ObsidianNoteProcessor::new(obsidian_notes);
    processor.load_content()?;
    println!("Loaded {} notes.", processor.notes().len());

    processor
        .load_content()?
        .rename_property(&property, &new_name)?;
    
    println!("Found {} matching notes", processor.modified_count());

    if cli.dry_run {
        println!("Dry run mode, not saving changes.");
        return Ok(())
    }

    processor.save()?;
    println!(
        "Renamed property {:?} to {:?} in {} notes.", property, new_name, processor.modified_count());
    return Ok(())
}
