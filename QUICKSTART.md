# Quick Start Guide

## Installation (One-Time Setup)

```bash
cd /Volumes/Phobos/projects/home_video_loader
./install.sh
```

The install script will:
1. Build the `tvcode` binary
2. Offer installation options
3. Set up PATH access

**Recommended:** Choose option 1 to install to `/usr/local/bin`

## Usage

```bash
# Navigate to any folder with videos
cd ~/Movies/vacation-2024

# Run the transcoder
tvcode
```

That's it! Your videos will be converted to Apple TV-compatible H.264/AAC/MP4 format.

## What Gets Created

For each video that needs conversion:
- Input: `family_dinner.mkv`
- Output: `family_dinner_appletv.mp4`

Original files are never modified.

## Requirements

- FFmpeg installed: `brew install ffmpeg` (macOS)
- Rust installed: https://rustup.rs/

## Examples

**Convert home videos:**
```bash
cd ~/Desktop/home-videos
tvcode
```

**Convert downloaded movies:**
```bash
cd ~/Downloads
tvcode
```

**Convert and organize:**
```bash
cd ~/Videos/unsorted
tvcode
# Review the _appletv.mp4 files
# Move them to your organized folder
mv *_appletv.mp4 ~/Videos/apple-tv/
```

## Output Quality

All videos are converted to:
- **Video:** H.264 High Profile (8-20 Mbps depending on resolution)
- **Audio:** AAC stereo at 192 kbps
- **Container:** MP4 with fast-start for streaming

## Speed

Hardware acceleration makes this very fast:
- **4K video:** ~2-5 minutes per hour of video
- **1080p video:** ~1-3 minutes per hour of video
- **720p video:** ~0.5-1 minute per hour of video

(Actual speed depends on your hardware)

## Troubleshooting

**Command not found:**
```bash
# Check if it's built
ls /Volumes/Phobos/projects/home_video_loader/target/release/tvcode

# If yes, run directly
/Volumes/Phobos/projects/home_video_loader/target/release/tvcode

# Or re-run install script
cd /Volumes/Phobos/projects/home_video_loader
./install.sh
```

**FFmpeg not found:**
```bash
brew install ffmpeg
```

## Uninstall

```bash
# If installed to /usr/local/bin
sudo rm /usr/local/bin/tvcode

# If added to PATH, remove the line from ~/.zshrc or ~/.bashrc
```
