# VidSpeed 🎬

> *Strip the audio. Change the speed. Get on with your life.*

---

## What Even Is This?

You know that feeling when someone sends you a 45-minute lecture recording and you have exactly 20 minutes before your meeting? Or when you're reviewing screen recordings of your own work and you have to sit through yourself slowly typing something you already know how to do? Or — and this one's personal — when someone shares a tutorial video that could've been a two-minute GIF but instead comes in at a breezy 38 minutes because the presenter pauses to think... a lot?

VidSpeed was built for that. Exactly that.

It's a command-line tool written in Rust that does two things and does them well:

1. **Changes the playback speed** of any video file — faster, slower, whatever you need.
2. **Removes the audio entirely** — because at 3× speed, audio is just noise anyway.

No GUI. No subscription. No cloud upload. No "processing on our servers." Your video stays on your machine, gets handed to FFmpeg (which does the actual heavy lifting), and comes out the other side ready to watch. That's the whole thing.

It's fast because Rust is fast. It's reliable because FFmpeg has been battle-tested for two decades. And it's simple because that's the point.

---

## Who Is This For?

- **Students** drowning in recorded lectures who need to get through material at 2× without losing their minds.
- **Developers** reviewing screen captures, demos, or QA recordings.
- **Content creators** who want to quickly preview footage at different speeds before editing.
- **Researchers** skimming through video data.
- **Anyone** who has ever right-clicked a video and wished "speed up" was just a thing you could do from the terminal.

If you're comfortable opening a terminal and have FFmpeg installed (more on that in a moment), you're the target audience.

---

## Prerequisites — The Stuff You Need Before You Start

Let's be honest upfront: VidSpeed does not do the video processing itself. It's a well-dressed wrapper around FFmpeg. So you need both things installed. Here's what you need:

### 1. Rust & Cargo

Rust is the programming language VidSpeed is written in. Cargo is Rust's package manager and build tool — it's bundled with Rust automatically, so you only need to install one thing.

**Install Rust (all platforms):**

Head to [https://rustup.rs](https://rustup.rs) and follow the instructions. On macOS and Linux it's literally one command:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

On Windows, download and run `rustup-init.exe` from the same site.

After installation, restart your terminal (or run `source ~/.cargo/env` on Unix), then verify:

```bash
rustc --version
cargo --version
```

You want Rust **1.70 or newer**. If you already have Rust installed and it's older than that, update it:

```bash
rustup update stable
```

### 2. FFmpeg

FFmpeg is the open-source multimedia framework that actually handles the video processing. It's been around since 2000, it supports basically every video format ever invented, and it is absolutely non-negotiable for VidSpeed to work.

**macOS:**

If you have Homebrew (and if you're a developer on macOS, you really should):

```bash
brew install ffmpeg
```

If you don't have Homebrew, get it first at [https://brew.sh](https://brew.sh), then run the above.

**Ubuntu / Debian:**

```bash
sudo apt update
sudo apt install ffmpeg
```

**Fedora / RHEL / CentOS:**

```bash
sudo dnf install ffmpeg
```

If FFmpeg isn't in your default repos (it sometimes isn't on RHEL-based systems due to licensing), enable RPM Fusion first:

```bash
sudo dnf install https://download1.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm
sudo dnf install ffmpeg
```

**Windows:**

Download a pre-built binary from [https://ffmpeg.org/download.html](https://ffmpeg.org/download.html) (the gyan.dev or BtbN builds are reliable). Extract the archive, and add the `bin` folder inside it to your system PATH. If you don't know how to add something to your PATH on Windows, search "add to PATH Windows 11" — it's a 30-second operation through System Properties.

Alternatively, if you use Windows Subsystem for Linux (WSL2), just install FFmpeg inside your Linux environment using the Ubuntu instructions above.

**Verify FFmpeg is installed and on your PATH:**

```bash
ffmpeg -version
```

You should see a wall of version and configuration text. If you get "command not found," FFmpeg is either not installed or not on your PATH. Fix that before proceeding — VidSpeed cannot do anything without it.

---

## Installation — Getting VidSpeed Onto Your Machine

### Clone the Repository

```bash
git clone https://github.com/yourusername/vidspeed.git
cd vidspeed
```

Or if you downloaded a zip, extract it and navigate into the folder.

### Set Up Your Environment File

VidSpeed reads a `.env` file for configuration. There's an example one included. Copy it:

```bash
cp .env.example .env
```

Or if you're starting fresh, create a `.env` file in the project root with the following content:

```env
# How many hours before processed files are considered old enough to clean up
TEMP_FILE_TTL_HOURS=24

# Log level — options: error, warn, info, debug, trace
RUST_LOG=info
```

You don't have to change anything here to get started. The defaults are sensible. But they're there if you want to tweak behaviour.

### Build the Project

```bash
cargo build --release
```

The first build will take a minute or two — Cargo is downloading and compiling all the dependencies. Subsequent builds are much faster because Cargo caches compiled artifacts.

When it's done, your binary will be sitting at:

```
target/release/vidspeed
```

### (Optional) Install It System-Wide

If you want to run `vidspeed` from anywhere on your system without specifying the full path, install it:

```bash
cargo install --path .
```

This puts the binary in `~/.cargo/bin/`, which should already be on your PATH if you installed Rust via rustup. After this you can just type `vidspeed` from any directory.

---

## Project Structure — What's What

```
vidspeed/
├── src/
│   ├── main.rs        # Entry point. Parses CLI arguments, validates input, hands off to cli.rs
│   ├── cli.rs         # Handles the user-facing output — the coloured text, the summary printout
│   └── processor.rs   # The engine room. Builds and runs the FFmpeg command. Handles cleanup logic.
├── Cargo.toml         # Project manifest. Dependencies, binary name, edition.
├── Cargo.lock         # Locked dependency versions. Committed to source control intentionally.
├── .env               # Your local configuration. Never commit this.
├── .env.example       # The template for .env. Safe to commit.
└── .gitignore         # Keeps build artifacts and secrets out of git.
```

There's no web server. There's no database. There's no background daemon. It's a binary you run, it does the thing, it exits. Clean.

---

## Usage — Actually Using the Thing

The basic shape of a VidSpeed command is:

```bash
vidspeed [OPTIONS] <INPUT>
```

### The Simplest Possible Use Case

```bash
vidspeed myvideo.mp4
```

This runs `myvideo.mp4` at the default speed of **2×** with audio removed. The output file is automatically named and saved in the same directory as the input:

```
myvideo_speed2x_noaudio.mp4
```

### Specifying a Speed

Use `-s` or `--speed` followed by a number:

```bash
# 1.5× faster
vidspeed myvideo.mp4 -s 1.5

# 3× faster
vidspeed myvideo.mp4 -s 3.0

# Half speed (slow motion)
vidspeed myvideo.mp4 -s 0.5

# Quarter speed
vidspeed myvideo.mp4 -s 0.25
```

Speed must be between **0.1** and **10.0**. Below 0.1 and FFmpeg starts producing results that are more art project than video. Above 10× and you're basically making a GIF. The tool will reject values outside this range with a clear error message.

A speed of exactly `1.0` will process the video at normal speed — useful if you just want to strip the audio without changing anything else.

### Specifying an Output File

Use `-o` or `--output` to control where the result goes:

```bash
vidspeed myvideo.mp4 -s 2.0 -o /path/to/output/finished.mp4
```

You can put the output anywhere on your filesystem that you have write access to. The directory must already exist — VidSpeed won't create nested directories for you.

### Putting It All Together

```bash
vidspeed /videos/lecture_recording.mp4 --speed 2.5 --output /videos/processed/lecture_fast.mp4
```

### Getting Help

```bash
vidspeed --help
```

This prints the full usage information directly in your terminal. Everything in this README about flags and arguments is also in there.

---

## What Happens When You Run It

Here's what VidSpeed actually does, step by step, so there are no mysteries:

1. **Parses your arguments.** Clap (the CLI argument library) reads what you typed and validates the structure.

2. **Validates the speed.** If you passed something outside 0.1–10.0, it stops here with an error.

3. **Checks the input file exists.** If the path is wrong or the file isn't there, it stops and tells you.

4. **Builds the FFmpeg command.** The core of the processing is this FFmpeg filter:
   ```
   setpts=(1/speed)*PTS
   ```
   `PTS` stands for Presentation Timestamp. By scaling it down, frames are presented sooner, making the video play faster. By scaling it up, frames are presented later, slowing it down. `-an` is added to the command to strip audio entirely.

5. **Runs FFmpeg.** The FFmpeg process is spawned as a child process. stdout and stderr are discarded (they go to `/dev/null` effectively) — FFmpeg's output is very verbose and not useful to you in normal operation. If you need to see it for debugging, set `RUST_LOG=debug` in your `.env`.

6. **Waits for completion.** A progress spinner runs while FFmpeg works. The actual progress percentage isn't tracked (FFmpeg's progress reporting would require parsing its stderr in real time, which adds complexity not worth it for a CLI tool). The spinner at least tells you it's alive and working.

7. **Reports success or failure.** If FFmpeg exits cleanly, you get a green checkmark and a success message. If it exits with a non-zero code, you get an error with the exit code and a suggestion to check FFmpeg directly.

---

## Configuration Reference

All configuration lives in your `.env` file. Here's every option:

| Variable | Default | Description |
|---|---|---|
| `RUST_LOG` | `info` | Log verbosity. Options: `error`, `warn`, `info`, `debug`, `trace` |
| `TEMP_FILE_TTL_HOURS` | `24` | Hours before a file is considered eligible for cleanup (used by the cleanup utility in `processor.rs`) |

That's it. VidSpeed is deliberately minimal in its configuration surface.

### About RUST_LOG

`RUST_LOG` controls how much the application prints to your terminal beyond the normal user-facing output. In practice:

- `error` — only fatal problems
- `warn` — problems that didn't stop execution
- `info` — normal operational messages (what you want day-to-day)
- `debug` — detailed internal flow, useful when something isn't working
- `trace` — everything, including things you probably don't care about

For debugging a problem, set `RUST_LOG=debug` in your `.env` and run again. You'll see the full FFmpeg command being built and any internal state.

---

## Supported Formats

VidSpeed accepts whatever FFmpeg accepts, which is essentially everything:

**Input formats that definitely work:**
- `.mp4` (H.264, H.265)
- `.mov` (QuickTime)
- `.avi`
- `.mkv` (Matroska)
- `.webm`
- `.flv`
- `.wmv`
- `.m4v`
- `.ts` (MPEG Transport Stream)
- `.3gp`

**Output format:**
Always `.mp4` with H.264 encoding (`libx264`). This is the most compatible format for playback on any device or platform. If your input is some exotic format, the output will still be a clean, standard `.mp4`.

---

## Troubleshooting — When Things Go Wrong

### "command not found: vidspeed"

You either haven't built the project yet, or you haven't installed it system-wide. Either run it with the full path:

```bash
./target/release/vidspeed myvideo.mp4
```

Or install it:

```bash
cargo install --path .
```

### "Failed to execute ffmpeg. Is ffmpeg installed and on PATH?"

FFmpeg is not installed, or it's installed but not on your PATH. Run `ffmpeg -version` in your terminal. If that fails, go back to the Prerequisites section and install FFmpeg properly.

### "Input file does not exist"

The path you provided doesn't point to a real file. Common causes:
- Typo in the filename
- File is in a different directory than you think
- Filename has spaces — wrap it in quotes: `vidspeed "my video file.mp4"`
- You're running from a different directory than where the file lives — use the absolute path

### "Video processing failed (ffmpeg exit code X)"

FFmpeg ran but encountered an error during processing. This usually means:

- The input file is corrupted or incomplete
- The output directory doesn't exist (create it manually first)
- You don't have write permission to the output location
- The input file is a format FFmpeg can't decode (rare, but possible with some DRM-protected files)

To get more detail, temporarily edit `processor.rs` and change both `Stdio::null()` lines to `Stdio::inherit()`, rebuild, and run again. You'll see FFmpeg's full output in your terminal, which will tell you exactly what went wrong.

### "Speed must be between 0.1 and 10.0"

Self-explanatory. Don't go below 0.1 or above 10.0. If you genuinely need speeds outside this range, you can modify the validation in `main.rs` — it's a one-line change.

### The output video has no audio

That's not a bug, that's the feature. Audio is always removed. The tool is called VidSpeed and the badge says "No Audio" for a reason. Speed-changed audio sounds terrible and is more confusing than helpful in most use cases. This is intentional and by design.

### Build fails with dependency errors

Make sure you're on a recent enough version of Rust:

```bash
rustup update stable
cargo clean
cargo build --release
```

`cargo clean` removes all cached build artifacts and forces a full rebuild. It takes longer but eliminates stale cache as a cause of weird build failures.

---

## How the Code Is Organised — A Brief Tour

### `main.rs`

The entry point. It does three things: initialise the logger, parse the CLI arguments using Clap, validate the speed value, then call into `cli::process_video`. Nothing clever happens here — it's intentionally thin. If you want to add a new subcommand or flag in the future, this is where it goes.

### `cli.rs`

This is the presentation layer. It's responsible for everything the user sees before and after processing — the coloured header, the input/output/speed summary, the success message. It constructs the file paths (handling the auto-naming logic when no output path is specified), creates the `VideoProcessor`, and calls it. If you want to change what VidSpeed prints to the terminal, this is where you look.

### `processor.rs`

This is where the actual work happens. The `VideoProcessor` struct holds the input path, output path, and speed. The `process()` method builds the FFmpeg command and runs it. The `process_with_progress()` method wraps that with an `indicatif` progress spinner.

There's also a `cleanup_old_files()` function in here — it walks a directory and removes files older than a given number of hours. This isn't called automatically in the CLI version, but it's there if you want to wire it up to a cron job or add a `--cleanup` flag in the future.

### `Cargo.toml`

The project manifest. It declares the binary name (`vidspeed`), the Rust edition (2021), and all dependencies. The dependency list is intentionally minimal for a CLI tool — Clap for argument parsing, Tokio for async runtime, indicatif for the progress bar, colored for terminal colours, FFmpeg via system command (no Rust FFmpeg bindings needed), tracing for logging, and a handful of utilities.

---

## Dependency Rationale — Why These Crates

You might wonder why a simple "run FFmpeg" tool uses an async runtime (Tokio). The reason is that spawning a child process and waiting for it is an I/O operation, and doing it asynchronously means the runtime can do other things while FFmpeg is crunching. It also makes it trivial to add concurrent processing in the future without restructuring the entire codebase.

**Clap** is the standard for CLI argument parsing in Rust. The derive feature means argument structs are defined with annotations rather than builder chains, which is cleaner to read and maintain.

**indicatif** gives the progress spinner. It's lightweight and well-maintained. The spinner isn't a true progress bar (we're not parsing FFmpeg's progress output) but it signals to the user that the process is alive and working, which matters for long videos.

**colored** handles terminal colour codes. It abstracts over platform differences so that the green checkmarks and red error markers work correctly on macOS, Linux, and Windows Terminal.

**anyhow** and **thiserror** (via anyhow's error handling) give structured error propagation with context. Instead of `.unwrap()` everywhere (which panics on failure with an unhelpful message), errors are collected with context about where they came from and returned to the user in a readable form.

**tracing** and **tracing-subscriber** provide structured logging. More powerful than `println!` for a production tool — log levels, filtering, and output formatting are all configurable without code changes.

---

## Performance Notes

VidSpeed uses FFmpeg's `libx264` encoder with `preset medium` and `crf 23`. These are balanced defaults that produce good quality at reasonable file sizes and encoding speed. Here's what each means:

- **`libx264`** — H.264 video encoding. Universally compatible. If you need H.265 for smaller file sizes, change `-c:v libx264` to `-c:v libx265` in `processor.rs` (note: encoding will be slower and the output may not play on older devices).

- **`preset medium`** — A speed/compression tradeoff. `ultrafast` encodes very quickly but produces larger files. `veryslow` takes much longer but produces smaller files at the same quality. `medium` is the sensible middle ground for most use cases.

- **`crf 23`** — Constant Rate Factor. Controls quality. Lower = better quality, larger file. Higher = worse quality, smaller file. Range is 0 (lossless) to 51 (worst). 23 is FFmpeg's own recommended default for good-looking output.

If you're processing many files or have specific size/quality requirements, these three values in `processor.rs` are the knobs to turn.

---

## Future Ideas — If This Ever Grows

A few directions this could go if the need arises:

- **Batch processing** — accept a directory as input and process every video in it.
- **Audio preservation with pitch correction** — some users may want to keep audio but corrected for the speed change. FFmpeg supports this via the `atempo` filter.
- **Progress percentage** — parse FFmpeg's progress output from stderr to show a real percentage rather than a spinner.
- **Format selection** — let users specify the output codec or container.
- **A `--dry-run` flag** — print what would happen without actually doing it.
- **Cleanup command** — expose the `cleanup_old_files` function as a `vidspeed clean` subcommand.

None of these are planned. But the codebase is small enough that any of them would be a few hours of work.

---

## Contributing

This is a small personal tool, but if you find a bug or have a meaningful improvement, the usual approach applies:

1. Fork the repository
2. Create a branch (`git checkout -b fix/the-thing`)
3. Make your changes
4. Test them against real video files
5. Open a pull request with a clear description of what changed and why

Please don't open pull requests that just add dependencies for things that can be done without them, or that significantly increase build complexity. The point of this tool is that it's small and understandable.

---

## License

MIT. Do what you want with it. Attribution appreciated but not required.

---

## A Final Word

VidSpeed exists because the gap between "I need this video at 2×" and "I have this video at 2×" was too many steps. Open a video editor, import the file, find the speed setting, export, wait for the export, done. For a task you might do ten times a day, that's genuinely painful.

Now it's one command. You're welcome.

```bash
vidspeed lecture.mp4 -s 2.0
```

Go watch something.
