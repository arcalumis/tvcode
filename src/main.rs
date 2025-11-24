use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// Structures for parsing ffprobe JSON output
#[derive(Debug, Deserialize, Serialize)]
struct FFProbeOutput {
    streams: Vec<Stream>,
    format: Format,
}

#[derive(Debug, Deserialize, Serialize)]
struct Stream {
    codec_type: String,
    codec_name: String,
    #[serde(default)]
    width: u32,
    #[serde(default)]
    height: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Format {
    format_name: String,
}

#[derive(Debug)]
struct VideoInfo {
    path: PathBuf,
    video_codec: String,
    audio_codec: String,
    container: String,
    width: u32,
    height: u32,
}

fn main() {
    println!("📺 tvcode - Apple TV Video Transcoder");
    println!("======================================\n");

    // Check if ffmpeg and ffprobe are available
    if !check_ffmpeg_installed() {
        eprintln!("❌ Error: ffmpeg and ffprobe must be installed and in PATH");
        eprintln!("   Install with: brew install ffmpeg (macOS)");
        std::process::exit(1);
    }

    // Get current directory
    let current_dir = env::current_dir().expect("Failed to get current directory");
    println!("📁 Scanning directory: {}\n", current_dir.display());

    // Find all video files
    let video_files = find_video_files(&current_dir);
    
    if video_files.is_empty() {
        println!("No video files found in the current directory.");
        return;
    }

    println!("Found {} video file(s)\n", video_files.len());

    // Process each video file
    for video_path in video_files {
        process_video(&video_path);
        println!(); // Add spacing between files
    }

    println!("✅ All done!");
}

fn check_ffmpeg_installed() -> bool {
    let ffmpeg_check = Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    
    let ffprobe_check = Command::new("ffprobe")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    ffmpeg_check.is_ok() && ffprobe_check.is_ok()
}

fn find_video_files(dir: &Path) -> Vec<PathBuf> {
    let video_extensions = [
        "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", 
        "m4v", "mpg", "mpeg", "3gp", "ts", "m2ts"
    ];

    let mut video_files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if let Some(ext_str) = extension.to_str() {
                        if video_extensions.contains(&ext_str.to_lowercase().as_str()) {
                            video_files.push(path);
                        }
                    }
                }
            }
        }
    }

    video_files
}

fn process_video(video_path: &Path) {
    println!("🎥 Processing: {}", video_path.file_name().unwrap().to_string_lossy());
    
    // Get video info using ffprobe
    match get_video_info(video_path) {
        Ok(info) => {
            println!("   Video: {} ({}x{})", info.video_codec, info.width, info.height);
            println!("   Audio: {}", info.audio_codec);
            println!("   Container: {}", info.container);

            // Check if transcoding is needed
            if needs_transcoding(&info) {
                println!("   ⚙️  Transcoding to H.264/AAC...");
                transcode_video(&info);
            } else {
                println!("   ✅ Already H.264/AAC Apple TV compatible, skipping");
            }
        }
        Err(e) => {
            eprintln!("   ❌ Error analyzing video: {}", e);
        }
    }
}

fn get_video_info(video_path: &Path) -> Result<VideoInfo, String> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            video_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run ffprobe: {}", e))?;

    if !output.status.success() {
        return Err("ffprobe failed".to_string());
    }

    let probe_data: FFProbeOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse ffprobe output: {}", e))?;

    let mut video_codec = String::from("unknown");
    let mut audio_codec = String::from("unknown");
    let mut width = 0;
    let mut height = 0;

    for stream in &probe_data.streams {
        match stream.codec_type.as_str() {
            "video" => {
                video_codec = stream.codec_name.clone();
                width = stream.width;
                height = stream.height;
            }
            "audio" => {
                audio_codec = stream.codec_name.clone();
            }
            _ => {}
        }
    }

    Ok(VideoInfo {
        path: video_path.to_path_buf(),
        video_codec,
        audio_codec,
        container: probe_data.format.format_name,
        width,
        height,
    })
}

fn needs_transcoding(info: &VideoInfo) -> bool {
    // Apple TV REQUIRES H.264 video and AAC audio in MP4 container
    // We explicitly check for these exact codecs
    
    let video_compatible = info.video_codec == "h264";  // Must be H.264, not HEVC
    let audio_compatible = info.audio_codec == "aac";   // Must be AAC
    let container_compatible = info.container.contains("mp4") || info.container.contains("m4v");

    !(video_compatible && audio_compatible && container_compatible)
}

fn transcode_video(info: &VideoInfo) {
    let output_path = get_output_path(&info.path);
    
    println!("   📤 Output: {}", output_path.file_name().unwrap().to_string_lossy());

    // Detect hardware acceleration based on platform
    let hw_accel = detect_hardware_acceleration();
    
    let mut ffmpeg_args = vec![
        "-i".to_string(),
        info.path.to_str().unwrap().to_string(),
    ];

    // Video encoding settings - ALWAYS H.264
    match &hw_accel {
        Some(hw) => {
            println!("   🚀 Using hardware acceleration: {} (H.264)", hw);
            ffmpeg_args.extend(get_hw_encoding_args(hw, info.width, info.height));
        }
        None => {
            println!("   ⚠️  Using software encoding (H.264, slower)");
            ffmpeg_args.extend(get_sw_encoding_args());
        }
    }

    // Audio encoding settings - ALWAYS AAC
    if info.audio_codec != "aac" {
        println!("   🔊 Converting audio to AAC");
        ffmpeg_args.extend([
            "-c:a".to_string(),
            "aac".to_string(),          // FORCE AAC codec
            "-b:a".to_string(),
            "192k".to_string(),          // High quality AAC at 192 kbps
            "-ac".to_string(),
            "2".to_string(),             // Stereo audio
        ]);
    } else {
        println!("   🔊 Audio already AAC, copying");
        ffmpeg_args.extend([
            "-c:a".to_string(),
            "copy".to_string(),
        ]);
    }

    // Output settings - MP4 container for Apple TV
    ffmpeg_args.extend([
        "-movflags".to_string(),
        "+faststart".to_string(),        // Enable streaming/fast start
        "-f".to_string(),
        "mp4".to_string(),               // FORCE MP4 container
        "-y".to_string(),                // Overwrite output file
        output_path.to_str().unwrap().to_string(),
    ]);

    println!("   🔄 Starting transcode...");

    // Run ffmpeg
    let status = Command::new("ffmpeg")
        .args(&ffmpeg_args)
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("   ✅ Transcode completed: H.264/AAC/MP4");
        }
        Ok(status) => {
            eprintln!("   ❌ Transcode failed with exit code: {:?}", status.code());
        }
        Err(e) => {
            eprintln!("   ❌ Failed to run ffmpeg: {}", e);
        }
    }
}

fn get_output_path(input_path: &Path) -> PathBuf {
    let stem = input_path.file_stem().unwrap().to_string_lossy();
    let parent = input_path.parent().unwrap();
    parent.join(format!("{}_appletv.mp4", stem))
}

fn detect_hardware_acceleration() -> Option<String> {
    // Try to detect platform and available hardware acceleration
    // All HW encoders will produce H.264
    
    #[cfg(target_os = "macos")]
    {
        // macOS - use VideoToolbox for H.264 encoding
        return Some("videotoolbox".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        // Windows - try NVIDIA NVENC first, then Intel QuickSync
        if check_encoder_available("h264_nvenc") {
            return Some("nvenc".to_string());
        } else if check_encoder_available("h264_qsv") {
            return Some("qsv".to_string());
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux - try NVIDIA NVENC, then VAAPI
        if check_encoder_available("h264_nvenc") {
            return Some("nvenc".to_string());
        } else if check_encoder_available("h264_vaapi") {
            return Some("vaapi".to_string());
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

// Only compile this function on Windows and Linux where it's actually used
#[cfg(any(target_os = "windows", target_os = "linux"))]
fn check_encoder_available(encoder: &str) -> bool {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.contains(encoder)
    } else {
        false
    }
}

fn get_hw_encoding_args(hw_type: &str, width: u32, height: u32) -> Vec<String> {
    // ALL hardware encoders produce H.264 output
    match hw_type {
        "videotoolbox" => {
            vec![
                "-c:v".to_string(),
                "h264_videotoolbox".to_string(),   // H.264 via VideoToolbox
                "-b:v".to_string(),
                calculate_bitrate(width, height),
                "-profile:v".to_string(),
                "high".to_string(),                // H.264 High Profile
                "-level".to_string(),
                "4.1".to_string(),                 // H.264 Level 4.1 (compatible)
                "-allow_sw".to_string(),
                "1".to_string(),
            ]
        }
        "nvenc" => {
            vec![
                "-c:v".to_string(),
                "h264_nvenc".to_string(),          // H.264 via NVENC
                "-preset".to_string(),
                "p7".to_string(),                  // Highest quality preset
                "-b:v".to_string(),
                calculate_bitrate(width, height),
                "-maxrate".to_string(),
                calculate_max_bitrate(width, height),
                "-profile:v".to_string(),
                "high".to_string(),                // H.264 High Profile
                "-level".to_string(),
                "4.1".to_string(),                 // H.264 Level 4.1
            ]
        }
        "qsv" => {
            vec![
                "-c:v".to_string(),
                "h264_qsv".to_string(),            // H.264 via QuickSync
                "-preset".to_string(),
                "veryslow".to_string(),
                "-b:v".to_string(),
                calculate_bitrate(width, height),
                "-profile:v".to_string(),
                "high".to_string(),                // H.264 High Profile
                "-level".to_string(),
                "4.1".to_string(),
            ]
        }
        "vaapi" => {
            vec![
                "-vaapi_device".to_string(),
                "/dev/dri/renderD128".to_string(),
                "-c:v".to_string(),
                "h264_vaapi".to_string(),          // H.264 via VAAPI
                "-b:v".to_string(),
                calculate_bitrate(width, height),
                "-profile:v".to_string(),
                "high".to_string(),                // H.264 High Profile
            ]
        }
        _ => get_sw_encoding_args(),
    }
}

fn get_sw_encoding_args() -> Vec<String> {
    // Software H.264 encoding with libx264
    vec![
        "-c:v".to_string(),
        "libx264".to_string(),         // H.264 software encoder
        "-preset".to_string(),
        "medium".to_string(),
        "-crf".to_string(),
        "20".to_string(),              // High quality (18-23 is good range)
        "-profile:v".to_string(),
        "high".to_string(),            // H.264 High Profile
        "-level".to_string(),
        "4.1".to_string(),             // H.264 Level 4.1 for compatibility
    ]
}

fn calculate_bitrate(width: u32, height: u32) -> String {
    // Calculate appropriate H.264 bitrate based on resolution
    let pixels = width * height;
    let bitrate = if pixels >= 3840 * 2160 {
        // 4K - high bitrate for H.264
        "20M"
    } else if pixels >= 1920 * 1080 {
        // 1080p
        "8M"
    } else if pixels >= 1280 * 720 {
        // 720p
        "5M"
    } else {
        // SD
        "3M"
    };
    bitrate.to_string()
}

fn calculate_max_bitrate(width: u32, height: u32) -> String {
    // Max bitrate for VBV buffer (1.5x the target bitrate)
    let pixels = width * height;
    let bitrate = if pixels >= 3840 * 2160 {
        "30M"
    } else if pixels >= 1920 * 1080 {
        "12M"
    } else if pixels >= 1280 * 720 {
        "7M"
    } else {
        "4M"
    };
    bitrate.to_string()
}
