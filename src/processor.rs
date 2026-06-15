
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use anyhow::{Context, Result};
use tracing::{info, error};
use indicatif::{ProgressBar, ProgressStyle};

pub struct VideoProcessor {
    input: PathBuf,
    output: PathBuf,
    speed: f64,
}

impl VideoProcessor {
    pub fn new(input: PathBuf, output: PathBuf, speed: f64) -> Self {
        Self { input, output, speed }
    }

    pub async fn process(&self) -> Result<()> {
        if !self.input.exists() {
            anyhow::bail!("Input file does not exist: {}", self.input.display());
        }

        if self.speed <= 0.0 || self.speed > 10.0 {
            anyhow::bail!("Speed must be between 0.1 and 10.0, got {}", self.speed);
        }

        info!(
            "Processing video: {} -> {} at {}x speed (no audio)",
            self.input.display(),
            self.output.display(),
            self.speed
        );

        // setpts=1/speed shrinks presentation timestamps → faster playback
        let video_filter = format!("setpts={}*PTS", 1.0 / self.speed);

        // IMPORTANT: stdout/stderr must NOT be Stdio::piped() unless you actively
        // read from them. FFmpeg writes a lot to stderr; a piped-but-unread fd
        // fills the OS pipe buffer and deadlocks the child process.
        // Use Stdio::null() to discard, or Stdio::inherit() to see it in the
        // terminal (useful for debugging).
        let input_str = self.input.to_str()
            .context("Input path contains invalid UTF-8")?;
        let output_str = self.output.to_str()
            .context("Output path contains invalid UTF-8")?;

        let status = Command::new("ffmpeg")
            .args([
                "-i", input_str,
                "-filter:v", &video_filter,
                "-an",          // strip audio
                "-c:v", "libx264",
                "-preset", "medium",
                "-crf", "23",
                "-y",           // overwrite without prompt
                output_str,
            ])
            .stdout(Stdio::null())  // <-- was Stdio::piped() → deadlock
            .stderr(Stdio::null())  // <-- was Stdio::piped() → deadlock
            .status()
            .await
            .context("Failed to execute ffmpeg. Is ffmpeg installed and on PATH?")?;

        if status.success() {
            info!("✓ Successfully processed: {}", self.output.display());
            Ok(())
        } else {
            let code = status.code().unwrap_or(-1);
            error!("FFmpeg exited with code {}", code);
            anyhow::bail!(
                "Video processing failed (ffmpeg exit code {}). \
                 Re-run with RUST_LOG=debug or set stderr to Stdio::inherit() to see ffmpeg output.",
                code
            )
        }
    }

    pub async fn process_with_progress(&self) -> Result<()> {
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message("Processing");

        let result = self.process().await;

        if result.is_ok() {
            pb.finish_with_message("✓ Complete!");
        } else {
            pb.finish_with_message("✗ Failed!");
        }

        result
    }
}

pub async fn cleanup_old_files(directory: &Path, max_age_hours: u64) -> Result<()> {
    let now = chrono::Utc::now();
    let mut removed = 0;

    if !directory.exists() {
        return Ok(());
    }

    for entry in walkdir::WalkDir::new(directory)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if let Ok(metadata) = entry.metadata() {
            if let Ok(modified) = metadata.modified() {
                let modified: chrono::DateTime<chrono::Utc> = modified.into();
                let age = now.signed_duration_since(modified);

                if age.num_hours() > max_age_hours as i64 {
                    if tokio::fs::remove_file(entry.path()).await.is_ok() {
                        removed += 1;
                    }
                }
            }
        }
    }

    if removed > 0 {
        info!(
            "Cleaned up {} old files from {}",
            removed,
            directory.display()
        );
    }

    Ok(())
}
