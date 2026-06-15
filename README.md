# VidSpeed

A CLI tool that speeds up videos and strips the audio. That's it.

### The annoyance

Actually, there's no annoyance, I'm building this tool specifically as a test towards YouTube automation and Footage surveillance.

### What this does

You give it a video file and a speed. It gives you back a new video that plays at that speed with no audio. The original file stays untouched.

```bash
vidspeed lecture.mp4 -s 2.0
```

The output lands in the same folder as lecture_speed2x_noaudio.mp4.

### Usage

The basic shape:

```bash
vidspeed [OPTIONS] <INPUT>
```

Pick a speed with -s:

```bash
vidspeed recording.mp4 -s 1.5   # faster
vidspeed slowmo.mp4 -s 0.5      # half speed
```

Speed must be between 0.1 and 10.0. Below that gets weird. Above that is basically a flipbook, lol 😂.

Pick an output location with -o:

```bash
vidspeed video.mkv -s 2.5 -o ~/Desktop/fast.mp4
```

Need a reminder? run vidspeed --help.

### Install

###### You need Rust and FFmpeg first.

Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

FFmpeg:

· macOS: brew install ffmpeg
· Ubuntu/Debian: sudo apt install ffmpeg
· Windows: grab a binary from ffmpeg.org and add it to your PATH

Verify with ffmpeg -version. If that Fails, STOP HERE!!! Make sure you have ffmpeg before you continue, else nothing will work, trust me bro.

Now get VidSpeed:

```bash
git clone https://github.com/oboobotenefiok/vidspeed
cd vidspeed
cargo build --release
```

Run it directly:

```bash
./target/release/vidspeed myvideo.mp4
```

Or install it to your PATH:

```bash
cargo install --path .
```

Now you can just type vidspeed from anywhere.

### What happens when you run it

The tool checks your speed is valid, makes sure the input file exists, then hands everything to FFmpeg. FFmpeg does the actual video crunching while a little gun loader (or how is it called?) shoots so you know it's not frozen. When it's done, you get a success message or an error telling you what broke.

Audio always gets removed. Sped up audio sounds like chipmunks(not dogs, they'd be dead, maybe) on caffeine and helps no one.

### Configuration

There's a .env file if you want to tweak things. Copy .env.example to .env and edit these:

· RUST_LOG - how much detail to print. Options: error, warn, info, debug, trace. Default is info.
· TEMP_FILE_TTL_HOURS - how many hours before a file is considered old enough to clean up. Default is 24.

You probably don't need to change either.

### Formats

Whatever FFmpeg accepts. That's basically everything: .mp4, .mov, .avi, .mkv, .webm, .flv, .wmv, .ts, .3gp.

Output is always .mp4 with H.264 encoding. Plays on anything.

### ERROR GUIDE:

"command not found: vidspeed" - You didn't install it. Run it directly with ./target/release/vidspeed or run cargo install --path .

"Failed to execute ffmpeg" - FFmpeg isn't installed or not on your PATH. Run ffmpeg -version to check.

"Input file does not exist" - Typo, wrong path, or the filename has spaces. Wrap it in quotes: vidspeed "my video.mp4"

"Video processing failed" - The input file might be corrupted, the output folder might not exist, or you don't have write permissions. Set RUST_LOG=debug in your .env to see FFmpeg's full error message.

The output has no audio - That's the point. Read the tool name again.

### Project layout

```
vidspeed/
  src/
    main.rs      - grabs your arguments, validates speed, calls the rest
    cli.rs       - prints the colored output you see in the terminal
    processor.rs - builds and runs the FFmpeg command
  Cargo.toml     - dependencies and project settings
  .env.example   - template for your local config

.../etc...

```

### Dependencies

Crate this side ... Why this side.
clap ... Parses command line arguments so you don't have to
tokio Async runtime ... Handles waiting for FFmpeg without blocking
indicatif ... Shows that shooting progress thing so you know it's working
colored ... Makes error messages red and success messages green
anyhow ... Error handling that doesn't make you write 20 lines of boilerplate

A note on performance

FFmpeg encodes with H.264 at medium preset and CRF 23. That's the sensible default balance between speed, file size, and quality. If you want to tweak it, edit processor.rs and change -c:v libx264, -preset medium, or -crf 23.

### The future

Probably nothing. This does one small thing and does it well. But if someone really wanted to add batch processing or real progress percentages, the codebase is small enough that it wouldn't be hard.

### License

MIT. Do whatever you want.

---


Feel free to submit issues and pull requests.

With love,

- Obot