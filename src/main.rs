use clap::Parser;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Parser, Debug)]
#[command(name = "tvcode")]
#[command(version)]
#[command(about = "Convert videos to Apple TV-compatible H.264/AAC format with subtitle burning")]
struct Args {
    /// Enable subtitle burning mode (prompts for subtitle selection)
    #[arg(short, long)]
    subtitles: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct FFProbeOutput {
    streams: Vec<Stream>,
    format: Format,
}

#[derive(Debug, Deserialize, Serialize)]
struct Stream {
    index: usize,
    codec_type: String,
    codec_name: String,
    #[serde(default)]
    width: u32,
    #[serde(default)]
    height: u32,
    #[serde(default)]
    tags: StreamTags,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct StreamTags {
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    title: Option<String>,
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
    subtitles: Vec<SubtitleTrack>,
}

#[derive(Debug, Clone)]
struct SubtitleTrack {
    subtitle_index: usize,  // Index among subtitle streams only (0, 1, 2...)
    codec: String,
    language: Option<String>,
    title: Option<String>,
    is_bitmap: bool,        // PGS, DVB, DVD subtitles are bitmap-based
}

fn main() {
    let args = Args::parse();
    
    println!("📺 tvcode v{} - Apple TV Video Transcoder", env!("CARGO_PKG_VERSION"));
    if args.subtitles {
        println!("🔥 Subtitle burning mode enabled");
    }
    println!("======================================\n");

    if !check_ffmpeg_installed() {
        eprintln!("❌ Error: ffmpeg and ffprobe must be installed and in PATH");
        eprintln!("   Install with: brew install ffmpeg (macOS)");
        std::process::exit(1);
    }

    let current_dir = env::current_dir().expect("Failed to get current directory");
    println!("📁 Scanning directory: {}\n", current_dir.display());

    let video_files = find_video_files(&current_dir);
    
    if video_files.is_empty() {
        println!("No video files found in the current directory.");
        return;
    }

    println!("Found {} video file(s)\n", video_files.len());

    for video_path in video_files {
        process_video(&video_path, args.subtitles);
        println!();
    }

    println!("✅ All done!");
}

fn check_ffmpeg_installed() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
        && Command::new("ffprobe")
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
}

fn find_video_files(dir: &Path) -> Vec<PathBuf> {
    let video_extensions = [
        "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm",
        "m4v", "mpg", "mpeg", "3gp", "ts", "m2ts",
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

fn process_video(video_path: &Path, burn_subtitles: bool) {
    println!(
        "🎥 Processing: {}",
        video_path.file_name().unwrap().to_string_lossy()
    );

    match get_video_info(video_path) {
        Ok(info) => {
            println!(
                "   Video: {} ({}x{})",
                info.video_codec, info.width, info.height
            );
            println!("   Audio: {}", info.audio_codec);
            println!("   Container: {}", info.container);

            if !info.subtitles.is_empty() {
                println!("   Subtitles: {} track(s) found", info.subtitles.len());
            }

            let selected_subtitle = if burn_subtitles && !info.subtitles.is_empty() {
                select_subtitle_track(&info.subtitles)
            } else {
                None
            };

            let needs_transcode = needs_transcoding(&info) || selected_subtitle.is_some();

            if needs_transcode {
                if selected_subtitle.is_some() {
                    println!("   ⚙️  Transcoding to H.264/AAC with burned subtitles...");
                } else {
                    println!("   ⚙️  Transcoding to H.264/AAC...");
                }
                transcode_video(&info, selected_subtitle);
            } else {
                println!("   ✅ Already H.264/AAC Apple TV compatible, skipping");
            }
        }
        Err(e) => {
            eprintln!("   ❌ Error analyzing video: {}", e);
        }
    }
}

fn is_bitmap_subtitle(codec: &str) -> bool {
    matches!(
        codec,
        "hdmv_pgs_subtitle" | "pgssub" | "dvd_subtitle" | "dvdsub" | "dvb_subtitle" | "dvbsub"
    )
}

fn select_subtitle_track(subtitles: &[SubtitleTrack]) -> Option<SubtitleTrack> {
    if subtitles.is_empty() {
        return None;
    }

    println!("\n   📝 Available subtitle tracks:");
    for (idx, sub) in subtitles.iter().enumerate() {
        let lang = sub.language.as_deref().unwrap_or("unknown");
        let title = sub.title.as_deref().unwrap_or("");
        let title_str = if !title.is_empty() {
            format!(" - {}", title)
        } else {
            String::new()
        };
        let sub_type = if sub.is_bitmap { "bitmap" } else { "text" };
        println!(
            "      [{}] {} ({}, {}){}",
            idx + 1,
            lang,
            sub.codec,
            sub_type,
            title_str
        );
    }
    println!("      [0] Skip subtitle burning");

    print!("\n   Select subtitle track [0-{}]: ", subtitles.len());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    if let Ok(choice) = input.trim().parse::<usize>() {
        if choice == 0 {
            return None;
        }
        if choice > 0 && choice <= subtitles.len() {
            return Some(subtitles[choice - 1].clone());
        }
    }

    println!("   ⚠️  Invalid selection, skipping subtitle burning");
    None
}

fn get_video_info(video_path: &Path) -> Result<VideoInfo, String> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            "-analyzeduration",
            "100000000",  // 100 seconds - helps with PGS detection
            "-probesize",
            "100000000",  // 100 MB
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
    let mut subtitles = Vec::new();
    let mut subtitle_stream_index = 0usize;

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
            "subtitle" => {
                let is_bitmap = is_bitmap_subtitle(&stream.codec_name);
                subtitles.push(SubtitleTrack {
                    subtitle_index: subtitle_stream_index,
                    codec: stream.codec_name.clone(),
                    language: stream.tags.language.clone(),
                    title: stream.tags.title.clone(),
                    is_bitmap,
                });
                subtitle_stream_index += 1;
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
        subtitles,
    })
}

fn needs_transcoding(info: &VideoInfo) -> bool {
    let video_compatible = info.video_codec == "h264";
    let audio_compatible = info.audio_codec == "aac";
    let container_compatible =
        info.container.contains("mp4") || info.container.contains("m4v");
    !(video_compatible && audio_compatible && container_compatible)
}

fn transcode_video(info: &VideoInfo, subtitle_track: Option<SubtitleTrack>) {
    let output_path = get_output_path(&info.path, subtitle_track.is_some());
    println!(
        "   📤 Output: {}",
        output_path.file_name().unwrap().to_string_lossy()
    );

    let hw_accel = detect_hardware_acceleration();

    
    let mut ffmpeg_args: Vec<String> = Vec::new();
    
    // Add analyzeduration and probesize for better stream detection
    ffmpeg_args.extend([
        "-analyzeduration".to_string(),
        "100000000".to_string(),
        "-probesize".to_string(),
        "100000000".to_string(),
    ]);
    
    // Input file
    ffmpeg_args.extend(["-i".to_string(), info.path.to_str().unwrap().to_string()]);

    // Handle subtitle burning based on type
    if let Some(ref track) = subtitle_track {
        if track.is_bitmap {
            // Bitmap subtitles (PGS, DVD, DVB) - use filter_complex with overlay
            println!("   🔥 Burning bitmap subtitles (PGS/DVD) using overlay filter");
            ffmpeg_args.extend([
                "-filter_complex".to_string(),
                format!("[0:v][0:s:{}]overlay", track.subtitle_index),
            ]);
            // Software encoding required for filter_complex
            ffmpeg_args.extend(get_sw_encoding_args());
        } else {
            // Text subtitles (SRT, ASS, SSA, etc.) - use subtitles filter
            println!("   🔥 Burning text subtitles using subtitles filter");
            let input_file = info
                .path
                .to_str()
                .unwrap()
                .replace('\\', "\\\\")
                .replace(':', "\\:")
                .replace("'", "'\\''");
            ffmpeg_args.extend([
                "-vf".to_string(),
                format!("subtitles='{}':si={}", input_file, track.subtitle_index),
            ]);
            // Software encoding required for vf filter
            ffmpeg_args.extend(get_sw_encoding_args());
        }
    } else {
        // No subtitles - can use hardware acceleration
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
    }

    // Audio encoding
    if info.audio_codec != "aac" {
        println!("   🔊 Converting audio to AAC");
        ffmpeg_args.extend([
            "-c:a".to_string(),
            "aac".to_string(),
            "-b:a".to_string(),
            "192k".to_string(),
            "-ac".to_string(),
            "2".to_string(),
        ]);
    } else {
        println!("   🔊 Audio already AAC, copying");
        ffmpeg_args.extend(["-c:a".to_string(), "copy".to_string()]);
    }

    // No subtitle streams in output (already burned into video)
    ffmpeg_args.push("-sn".to_string());

    // Output settings
    ffmpeg_args.extend([
        "-movflags".to_string(),
        "+faststart".to_string(),
        "-f".to_string(),
        "mp4".to_string(),
        "-y".to_string(),
        output_path.to_str().unwrap().to_string(),
    ]);

    println!("   🔄 Starting transcode...");

    let status = Command::new("ffmpeg").args(&ffmpeg_args).status();

    match status {
        Ok(status) if status.success() => {
            if subtitle_track.is_some() {
                println!("   ✅ Transcode completed: H.264/AAC/MP4 with burned subtitles");
            } else {
                println!("   ✅ Transcode completed: H.264/AAC/MP4");
            }
        }
        Ok(status) => {
            eprintln!(
                "   ❌ Transcode failed with exit code: {:?}",
                status.code()
            );
        }
        Err(e) => {
            eprintln!("   ❌ Failed to run ffmpeg: {}", e);
        }
    }
}

fn get_output_path(input_path: &Path, has_subtitles: bool) -> PathBuf {
    let stem = input_path.file_stem().unwrap().to_string_lossy();
    let parent = input_path.parent().unwrap();
    if has_subtitles {
        parent.join(format!("{}_appletv_subs.mp4", stem))
    } else {
        parent.join(format!("{}_appletv.mp4", stem))
    }
}

fn detect_hardware_acceleration() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        return Some("videotoolbox".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        if check_encoder_available("h264_nvenc") {
            return Some("nvenc".to_string());
        } else if check_encoder_available("h264_qsv") {
            return Some("qsv".to_string());
        }
    }

    #[cfg(target_os = "linux")]
    {
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
    match hw_type {
        "videotoolbox" => vec![
            "-c:v".to_string(),
            "h264_videotoolbox".to_string(),
            "-b:v".to_string(),
            calculate_bitrate(width, height),
            "-profile:v".to_string(),
            "high".to_string(),
            "-level".to_string(),
            "4.1".to_string(),
            "-allow_sw".to_string(),
            "1".to_string(),
        ],
        "nvenc" => vec![
            "-c:v".to_string(),
            "h264_nvenc".to_string(),
            "-preset".to_string(),
            "p7".to_string(),
            "-b:v".to_string(),
            calculate_bitrate(width, height),
            "-maxrate".to_string(),
            calculate_max_bitrate(width, height),
            "-profile:v".to_string(),
            "high".to_string(),
            "-level".to_string(),
            "4.1".to_string(),
        ],
        "qsv" => vec![
            "-c:v".to_string(),
            "h264_qsv".to_string(),
            "-preset".to_string(),
            "veryslow".to_string(),
            "-b:v".to_string(),
            calculate_bitrate(width, height),
            "-profile:v".to_string(),
            "high".to_string(),
            "-level".to_string(),
            "4.1".to_string(),
        ],
        "vaapi" => vec![
            "-vaapi_device".to_string(),
            "/dev/dri/renderD128".to_string(),
            "-c:v".to_string(),
            "h264_vaapi".to_string(),
            "-b:v".to_string(),
            calculate_bitrate(width, height),
            "-profile:v".to_string(),
            "high".to_string(),
        ],
        _ => get_sw_encoding_args(),
    }
}

fn get_sw_encoding_args() -> Vec<String> {
    vec![
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "medium".to_string(),
        "-crf".to_string(),
        "20".to_string(),
        "-profile:v".to_string(),
        "high".to_string(),
        "-level".to_string(),
        "4.1".to_string(),
    ]
}

fn calculate_bitrate(width: u32, height: u32) -> String {
    let pixels = width * height;
    if pixels >= 3840 * 2160 {
        "20M".to_string()
    } else if pixels >= 1920 * 1080 {
        "8M".to_string()
    } else if pixels >= 1280 * 720 {
        "5M".to_string()
    } else {
        "3M".to_string()
    }
}

fn calculate_max_bitrate(width: u32, height: u32) -> String {
    let pixels = width * height;
    if pixels >= 3840 * 2160 {
        "30M".to_string()
    } else if pixels >= 1920 * 1080 {
        "12M".to_string()
    } else if pixels >= 1280 * 720 {
        "7M".to_string()
    } else {
        "4M".to_string()
    }
}
