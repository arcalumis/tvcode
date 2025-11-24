# tvcode - Apple TV Video Transcoder 📺

A lightning-fast Rust command-line tool that converts any video to Apple TV-compatible format using FFmpeg with hardware acceleration.

**Command:** `tvcode`

## What It Does

Run `tvcode` in any folder and it will:
- 🔍 Find all video files in the current directory
- 📊 Check their codec information
- ⚡ Convert them to **H.264 video + AAC audio** in MP4 format
- 🚀 Use hardware acceleration for maximum speed
- ✅ Skip files that are already compatible

**Output Format (Apple TV Compatible):**
- Video: H.264 (High Profile, Level 4.1)
- Audio: AAC (192 kbps stereo)
- Container: MP4 with fast-start flag
- Quality: Resolution-appropriate bitrates (3M-20M)

## Installation

### Prerequisites

Install FFmpeg:
```bash
# macOS
brew install ffmpeg

# Linux (Ubuntu/Debian)
sudo apt install ffmpeg

# Windows
choco install ffmpeg
```

### Install tvcode

1. **Build the binary:**
```bash
cd /Volumes/Phobos/projects/home_video_loader
cargo build --release
```

2. **Install to your PATH:**
```bash
# Copy to /usr/local/bin (macOS/Linux)
sudo cp target/release/tvcode /usr/local/bin/

# Or add to your PATH in ~/.zshrc or ~/.bashrc
export PATH="/Volumes/Phobos/projects/home_video_loader/target/release:$PATH"
```

3. **Verify installation:**
```bash
tvcode
```

## Usage

Just navigate to any folder with videos and run:

```bash
tvcode
```

That's it! The command will:
- Scan for video files (.mp4, .mkv, .avi, .mov, etc.)
- Show what it found
- Convert incompatible files to H.264/AAC/MP4
- Create new files with `_appletv.mp4` suffix

### Example Output

```
📺 tvcode - Apple TV Video Transcoder
======================================

📁 Scanning directory: /Users/you/Videos

Found 2 video file(s)

🎥 Processing: movie.mkv
   Video: hevc (3840x2160)
   Audio: ac3
   Container: matroska,webm
   ⚙️  Transcoding to H.264/AAC...
   📤 Output: movie_appletv.mp4
   🚀 Using hardware acceleration: videotoolbox (H.264)
   🔊 Converting audio to AAC
   🔄 Starting transcode...
   ✅ Transcode completed: H.264/AAC/MP4

🎥 Processing: video.mp4
   Video: h264 (1920x1080)
   Audio: aac
   Container: mov,mp4,m4a,3gp,3g2,mj2
   ✅ Already H.264/AAC Apple TV compatible, skipping

✅ All done!
```

## Hardware Acceleration

`tvcode` automatically detects and uses the fastest available encoder:

| Platform | First Choice | Second Choice | Fallback |
|----------|-------------|---------------|----------|
| **macOS** | VideoToolbox | — | libx264 |
| **Windows** | NVIDIA NVENC | Intel QuickSync | libx264 |
| **Linux** | NVIDIA NVENC | VAAPI | libx264 |

Hardware encoding is typically **5-10x faster** than software encoding.

## Supported Input Formats

- MP4, MKV, AVI, MOV, WMV
- FLV, WebM, M4V, MPG, MPEG
- 3GP, TS, M2TS

## Quality Settings

Bitrates are automatically chosen based on resolution:

| Resolution | Bitrate | Max Bitrate |
|------------|---------|-------------|
| 4K (2160p) | 20 Mbps | 30 Mbps |
| 1080p | 8 Mbps | 12 Mbps |
| 720p | 5 Mbps | 7 Mbps |
| SD | 3 Mbps | 4 Mbps |

All outputs use:
- H.264 High Profile (Level 4.1)
- AAC audio at 192 kbps
- CRF 20 for software encoding (excellent quality)

## Why H.264?

While Apple TV supports H.265/HEVC, H.264 offers:
- ✅ Universal compatibility across all Apple TV models
- ✅ Better hardware decoder support
- ✅ Faster encoding with hardware acceleration
- ✅ Excellent quality-to-size ratio
- ✅ No compatibility issues with older devices

## Tips

**Convert a specific folder:**
```bash
cd ~/Downloads/vacation-videos
tvcode
```

**Process videos and delete originals:**
```bash
# Review the output files first, then:
rm *_original_filename.mkv
```

**Check what will be converted (dry run):**
Just run `tvcode` - it will show what needs converting before doing anything.

## Troubleshooting

**"ffmpeg and ffprobe must be installed"**
- Install FFmpeg: `brew install ffmpeg` (macOS)
- Verify: `ffmpeg -version` and `ffprobe -version`

**Slow encoding:**
- Check if hardware acceleration is detected in the output
- macOS should always use VideoToolbox automatically
- Update GPU drivers on Windows/Linux

**Command not found:**
- Make sure the binary is in your PATH
- Try: `which tvcode` to see if it's installed
- Re-run the installation steps

## Files Created

- Original files are **never modified**
- Output files: `{original_name}_appletv.mp4`
- Outputs appear in the same directory as source files

## License

MIT License - use freely!
