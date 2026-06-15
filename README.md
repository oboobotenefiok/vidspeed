# Video Speed Changer

This is a tool that speeds up video files and removes the audio track entirely. You can use it from the command line for individual files or run it as a web server where people can upload videos through a browser interface. The project is written in Rust and uses ffmpeg to do the actual video processing.

I wrote this because I often need to speed up videos for quick reviews or create time-lapse effects, but I almost never want to keep the original audio. The tool handles both use cases in one package.

### What It Does

When you give it a video file and a speed factor, it does two things. First, it speeds up the video by dropping or duplicating frames according to the factor you choose. Second, it completely removes any audio streams from the output file. You end up with a silent video that plays back at the speed you requested.

The speed factor works like this. A factor of 2.0 makes the video play twice as fast, so a one minute video becomes thirty seconds. A factor of 0.5 makes it play at half speed, so that same one minute video stretches to two minutes. The tool accepts any factor between 0.1 and 10.0, though going to extremes might make the video look strange.

### How to Install

You need to have ffmpeg installed on your system first. The tool calls out to ffmpeg to do the actual processing, so it won't work without it.

On Ubuntu or Debian, run this:

sudo apt update
sudo apt install ffmpeg

On macOS with Homebrew:

brew install ffmpeg

On Windows, download ffmpeg from the official website and add it to your PATH.

Once ffmpeg is installed, you can build this tool from source. Make sure you have Rust installed. If you don't, go to rustup.rs and follow the instructions there.

Then clone the repository and build it:

git clone <repository-url>
cd vidspeed
cargo build --release

The binary will be in target/release/vidspeed. You can copy it somewhere in your PATH if you want to run it from anywhere.

Using the Command Line Tool

The command line interface is straightforward. You give it an input file, a speed factor, and optionally an output file name.

Here is the basic command:

vidspeed process -i video.mp4 -s 2.0

This takes video.mp4, speeds it up to twice the original speed, removes the audio, and saves the result as video_speed2x_noaudio.mp4 in the same folder.

If you want to specify the output file name, use the -o flag:

vidspeed process -i video.mp4 -s 1.5 -o fast_clip.mp4

For slow motion, use a factor less than one:

vidspeed process -i action.mov -s 0.5 -o slow_motion.mp4

The tool shows you what it is doing while it works. You will see a progress bar and some information about the input and output files. When it finishes, it tells you where the new file is located.

If you forget to provide an output file name, the tool makes one up for you based on the input file name and the speed factor you chose. This is convenient when you are processing many files and don't want to think about naming each one.

Running the Web Server

The web server mode lets people upload videos through a browser, choose a speed factor, and download the processed result. This is useful if you want to offer this functionality to others without making them install anything.

To start the web server, run:

vidspeed server --port 3000 --host 127.0.0.1

The host parameter tells the server which network interface to listen on. Using 127.0.0.1 makes it only accessible from your own computer. If you want to allow connections from other devices on your network, use 0.0.0.0 instead.

Once the server is running, open a web browser and go to http://localhost:3000. You will see a page with an upload area, a slider to choose the speed factor, and a button to start processing.

The upload area accepts MP4, AVI, MOV, and MKV files up to 500 megabytes. You can either click inside the area to select a file from your computer or drag and drop a file onto it.

After you select a file and choose a speed, click the Process Video button. The server uploads your file, starts processing it in the background, and shows you a progress bar. When processing finishes, a download link appears. Click it to get your sped-up, silent video.

The server keeps track of multiple jobs at once. If several people upload videos at the same time, the server processes them in parallel up to a limit you can configure. By default it handles four concurrent jobs. Additional uploads wait in a queue until a processing slot opens up.

Configuration

The web server reads configuration from environment variables or a .env file. Here are the settings you can change.

MAX_FILE_SIZE_MB controls how large uploaded videos can be. The default is 500 megabytes. If you expect larger files, increase this number.

MAX_CONCURRENT_JOBS sets how many videos the server processes at the same time. The default is 4. Setting this too high might overwhelm your server's CPU, especially for longer videos.

TEMP_FILE_TTL_HOURS determines how long the server keeps uploaded and processed files before deleting them. The default is 24 hours. After this time passes, the server automatically removes old files to free up disk space.

UPLOAD_DIR and PROCESSED_DIR tell the server where to store temporary files. The default is ./uploads and ./processed in the directory where you run the server.

RUST_LOG controls how much detail the server prints to its logs. Set it to debug for troubleshooting or info for normal operation.

To use a .env file, create a file named .env in the same directory as the server and put your settings there, one per line. The server reads this file when it starts.

How It Works Under the Hood

When you run the command line tool or upload a video through the web interface, the program builds an ffmpeg command that looks something like this:

ffmpeg -i input.mp4 -filter:v setpts=0.5*PTS -an -c:v libx264 -preset medium -crf 23 -y output.mp4

The setpts filter changes the presentation timestamps of the video frames. Dividing the PTS by the speed factor makes the frames play back faster because they are closer together in time.

The -an flag tells ffmpeg to exclude all audio streams. This is where the audio removal happens. There is no fancy audio processing or pitch preservation because there is no audio at all in the output.

The video codec is set to libx264, which produces good quality at reasonable file sizes. The preset medium balances encoding speed against compression efficiency, and the crf value of 23 gives decent quality without making the file too large.

The web server uses asynchronous processing so that the browser does not have to wait while ffmpeg churns through a large video. When you upload a file, the server immediately returns a job identifier and starts processing in the background. Your browser then polls the server every two seconds to check on the job's progress.

For simplicity, the web server stores job information in memory. This means that if you restart the server, all current jobs are lost. For a production deployment handling important data, you would want to add a persistent queue using Redis or a database. The code includes commented sections for this purpose.

Deploying the Web Server

Because this is a Rust program that runs continuously, you cannot host it on static site hosting like Netlify or GitHub Pages. You need a platform that runs server processes.

The easiest free option is Render. You create an account, connect your GitHub repository, and create a new Web Service. Render detects the Dockerfile automatically and builds the project. You set the start command to vidspeed server --host 0.0.0.0 --port 10000, and Render assigns you a public URL.

Fly.io is another good option. You install their command line tool, run fly launch in your project directory, and answer a few questions. Fly.io builds a Docker container and deploys it to their global network. Your app gets a URL like your-app-name.fly.dev.

If you prefer to run your own server, you can use Docker Compose. The project includes a docker-compose.yml file that sets up both the vidspeed server and a Redis container. Run docker-compose up -d, and everything starts in the background on port 3000.

For any deployment, make sure ffmpeg is installed in the environment. The Dockerfile already includes it, so that is taken care of when you use the container.

### Limitations and Considerations

The tool removes audio completely. If you need to keep the audio or preserve its pitch while speeding up, this is not the right tool for you. There are other projects that handle that scenario.

Very high speed factors, like 8x or 10x, may produce jerky video because ffmpeg has to drop many frames. Most video players handle this fine, but the result might not look smooth.

Very low speed factors, like 0.1x, will create enormous files because ffmpeg duplicates frames to stretch the video. The tool does not warn you about this, so be careful when using extreme slow motion.

The web server stores files in temporary directories and cleans them up after the time to live expires. However, if someone uploads a very large file and the server runs out of disk space, processing will fail. Monitor your disk usage if you expect many large uploads.

The in-memory job storage means that if the server crashes or restarts, all pending jobs disappear. For a public-facing service, you would want to implement persistent storage.

### Troubleshooting

If the command line tool gives you an error about ffmpeg not found, make sure ffmpeg is installed and accessible in your PATH. You can test this by typing ffmpeg -version in your terminal.

If the web server fails to start because the port is already in use, change the port number with the --port flag. Port 3000 is common, so something else might be using it.

If uploads fail with a file too large error, increase MAX_FILE_SIZE_MB in your .env file or environment settings. The server checks file size before processing, so this is a hard limit.

If processing seems to hang forever, check the server logs. The RUST_LOG environment variable controls log detail. Set it to debug to see what ffmpeg is doing.

If the progress bar in the web interface stops moving, the browser might have lost the connection to the server. Refresh the page and check the job list. Completed jobs appear there with download links even if you navigated away from the page.

### Building From Source

If you want to modify the code or just build it yourself, make sure you have Rust and Cargo installed. The project uses a standard Cargo layout.

Clone the repository and run cargo build --release. The binary ends up in target/release/vidspeed.

If you run into compilation errors about missing dependencies, try cargo clean and then cargo build again. Sometimes the build cache gets into a strange state.

The code uses several Rust crates for the web server, command line parsing, logging, and asynchronous processing. All of these are specified in Cargo.toml and download automatically when you build.

### Final Thoughts

This tool solves a specific problem well. It makes videos faster and removes the audio without any fuss. The dual command line and web interface give you flexibility in how you use it.

I have used it to quickly review security camera footage, create time-lapse videos of construction projects, and speed up lecture recordings to save time. In every case, not having to deal with audio simplified the process.

If you run into trouble or have ideas for improvement, feel free to open an issue on the repository. I cannot promise to fix everything, but I do read the reports.



With Love,

- Obot
