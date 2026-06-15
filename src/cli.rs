use std::path::PathBuf;
use anyhow::Result;
use colored::*;
use crate::processor::VideoProcessor;

pub async fn process_video(input: &str, output: Option<&str>, speed: f64) -> Result<()> {
    println!("{}", "=== Video Speed Changer (No Audio) ===".bright_blue().bold());
    println!();
    
    let input_path = PathBuf::from(input);
    
    if !input_path.exists() {
        eprintln!("{} Input file not found: {}", "✗".red(), input);
        std::process::exit(1);
    }
    
    let output_path = match output {
        Some(path) => PathBuf::from(path),
        None => {
            let stem = input_path.file_stem().unwrap_or_default();
            let parent = input_path.parent().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
            parent.join(format!("{}_speed{}x_noaudio.mp4", stem.to_string_lossy(), speed))
        }
    };
    
    println!("Input:  {}", input_path.display().to_string().cyan());
    println!("Output: {}", output_path.display().to_string().yellow());
    println!("Speed:  {}x {}", speed.to_string().green(), 
             if speed > 1.0 { "(faster)".bright_green() } else if speed < 1.0 { "(slower)".yellow() } else { "(normal)".white() });
    println!("Audio:  {}", "REMOVED".red().bold());
    println!();
    
    let processor = VideoProcessor::new(input_path, output_path, speed);
    processor.process_with_progress().await?;
    
    println!();
    println!("{} Video processed successfully!", "✓".green().bold());
    
    Ok(())
}
