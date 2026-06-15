You know those long videos you don't have time to watch? Lecture recordings, security footage, someone's slow-paced tutorial. VidSpeed is for those moments.

It's a simple command line tool written in Rust. It speeds up or slows down any video file and removes the audio, because sped up audio just sounds terrible anyway. That's it.

The video never leaves your machine. VidSpeed hands it to FFmpeg, which does the real work, and you get a new file ready to watch.

---

Who actually uses this?

Students binge watching recorded lectures at double speed. Developers reviewing screen recordings. Anyone comfortable opening a terminal and running a single command.

---

What you need first

You need two things installed before VidSpeed will work.

1. Rust and Cargo

Go to rustup.rs and follow the install instructions. After it's done, restart your terminal and run:

```bash
rustc --version
cargo --version
```

You want Rust 1.70 or newer. Update with rustup update stable if yours is older.

2. FFmpeg

On macOS with Homebrew:

```bash
brew install ffmpeg
```

On Ubuntu or Debian:

```bash
sudo apt update
sudo apt install ffmpeg
```

On Windows, grab a pre built binary from ffmpeg.org and add its bin folder to your system PATH.

Verify it works:

```bash
ffmpeg -version
```

If you see "command not found", stop here and fix that first.

---

Getting VidSpeed installed

Clone the repo and move into the folder:

```bash
git clone https://github.com/yourusername/vidspeed.git
cd vidspeed
```

Copy the example environment file:

```bash
cp .env.example .env
```

Then build it:

```bash
cargo build --release
```

Your binary ends up at target/release/vidspeed. To run it from anywhere, install it system wide:

```bash
cargo install --path .
```

---

How to actually use it

The basic command looks like this:

```bash
vidspeed myvideo.mp4
```

That runs your video at 2x speed with no audio. Output is saved in the same folder as myvideo_speed2x_noaudio.mp4.

To pick a different speed:

```bash
vidspeed myvideo.mp4 -s 1.5
vidspeed myvideo.mp4 -s 0.5   # slow motion
```

Speed must be between 0.1 and 10. Anything outside that gets rejected.

To choose where the output goes:

```bash
vidspeed myvideo.mp4 -s 2.0 -o /path/to/finished.mp4
```

Need help? Run vidspeed --help.

---

What happens when you run it

VidSpeed checks your speed value, makes sure the input file exists, builds an FFmpeg command with a filter that changes the video timing, strips the audio, runs the command with a progress spinner so you know it's working, then tells you if it succeeded or failed.

That's the whole loop.

---

Configuration

All settings live in a .env file in the project root. You have two options:

RUST_LOG controls how much detail prints to the terminal. Options are error, warn, info, debug, trace. Default is info.

TEMP_FILE_TTL_HOURS sets how many hours before a file is considered old enough to clean up. Default is 24.

---

Formats

VidSpeed accepts whatever FFmpeg accepts. That means basically everything: .mp4, .mov, .avi, .mkv, .webm, .flv, .wmv, .m4v, .ts, .3gp. The output is always a standard .mp4 file using H.264 encoding.

---

Common problems

"command not found: vidspeed"

You either haven't built it or haven't installed it. Run it directly with ./target/release/vidspeed myvideo.mp4 or install it with cargo install --path ..

"Failed to execute ffmpeg"

FFmpeg isn't installed or isn't on your PATH. Run ffmpeg -version to check.

"Input file does not exist"

Double check the path. If the filename has spaces, wrap it in quotes. Use an absolute path if you're in a different directory.

"Video processing failed"

This usually means the input file is corrupted, the output folder doesn't exist, or you don't have write permissions. Run with RUST_LOG=debug in your .env file to see more details.

The output has no audio

That's intentional. Audio always gets removed. The tool is called VidSpeed for a reason.

---

One last thing

VidSpeed exists because opening a video editor just to change playback speed is too many steps for something you might do ten times a day. Now it's one command.

```bash
vidspeed lecture.mp4 -s 2.0
```

I had initial intentions to make this work on the web too but the energy isn't really there for now. This actually has solved the problem and I'll only work towards modifying it to suit my current workflow if need be.

Feel free to submit and issue or Pull Request. I do not guarantee they will be attended to but I'll be looking at them.

Thanks for reading this far.

With Love,

- Obot