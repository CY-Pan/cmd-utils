use std::process::Command;

#[derive(Debug, Default)]
pub struct VideoInfo {
    pub vcodec: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub duration: f64,
    pub bitrate: u32,
    pub pixfmt: String,
    pub acodec: String,
}

pub fn probe_video(video_path: &str) -> VideoInfo {
    let v_probe_out = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_name,width,height,pix_fmt,r_frame_rate,duration,bit_rate",
            "-of",
            "default=nw=1:nk=1",
            video_path,
        ])
        .output()
        .unwrap();

    let out_str = String::from_utf8(v_probe_out.stdout).unwrap();
    let mut lines = out_str.lines();

    let vcodec: String = lines.next().unwrap().parse().unwrap();
    let width: u32 = lines.next().unwrap().parse().unwrap();
    let height: u32 = lines.next().unwrap().parse().unwrap();
    let pixfmt: String = lines.next().unwrap().parse().unwrap();

    let fps: f64 = {
        let fps: String = lines.next().unwrap().parse().unwrap();
        let mut line = fps.split('/');
        let numerator: f64 = line.next().unwrap().parse().unwrap();
        let denominator: f64 = line.next().unwrap().parse().unwrap();
        numerator / denominator
    };

    let duration: f64 = lines.next().unwrap().parse().unwrap();
    let bitrate: u32 = lines.next().unwrap().parse().unwrap();

    let a_probe_out = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=codec_name",
            "-of",
            "default=nw=1:nk=1",
            video_path,
        ])
        .output()
        .unwrap();

    let acodec = String::from_utf8(a_probe_out.stdout)
        .unwrap()
        .trim()
        .to_string();

    VideoInfo {
        vcodec,
        width,
        height,
        fps,
        duration,
        bitrate,
        pixfmt,
        acodec,
    }
}

#[derive(Debug, Default)]
pub struct VideoEncodeInfo {
    pub only_copy: bool,
    pub hardware_encode: bool,
    pub bitrate: u32,
    pub frame_rate: Option<u32>,
    pub pixfmt: Option<String>,
    pub clip_config: Option<VideoClipConfig>,
    pub crop_config: Option<VideoCropConfig>,
}

impl VideoEncodeInfo {
    pub fn hardware_default() -> Self {
        Self {
            hardware_encode: true,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default)]
pub struct VideoClipConfig {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Default)]
pub struct VideoCropConfig {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub fn calculate_target_bitrate(video_info: &VideoInfo) -> u32 {
    ((2e6 * video_info.fps) as u64 * (video_info.width * video_info.height) as u64 / 22_118_400)
        as u32
}

pub fn reencode_video(infile: &str, outfile: &str, encode_info: VideoEncodeInfo) {
    let mut args = vec!["-v", "warning", "-stats", "-i", infile];

    let bitrate_str = encode_info.bitrate.to_string();
    if encode_info.only_copy {
        args.extend(["-c", "copy"]);
    } else {
        if encode_info.bitrate > 0 {
            args.extend(["-b:v", bitrate_str.as_str()]);
        }
        if encode_info.hardware_encode {
            args.extend([
                "-c:v",
                if cfg!(target_os = "macos") {
                    "hevc_videotoolbox"
                } else {
                    "hevc_nvenc"
                },
            ]);
        }
    }

    let frame_rate_str;
    if let Some(frame_rate) = encode_info.frame_rate {
        frame_rate_str = frame_rate.to_string();
        args.extend(["-r", frame_rate_str.as_str()]);
    }

    if let Some(pixfmt) = &encode_info.pixfmt {
        args.extend(["-pix_fmt", pixfmt]);
    }

    if let Some(clip_config) = &encode_info.clip_config {
        if let Some(from) = &clip_config.from {
            args.extend(["-ss", from]);
        }
        if let Some(to) = &clip_config.to {
            args.extend(["-to", to]);
        }
    }

    let vf_arg;
    if let Some(crop_config) = encode_info.crop_config {
        vf_arg = format!(
            "crop={}:{}:{}:{}",
            crop_config.width, crop_config.height, crop_config.x, crop_config.y
        );
        args.extend(["-vf", vf_arg.as_str()]);
    }

    args.push(outfile);
    println!("   ffmpeg {}", args.join(" "));

    Command::new("ffmpeg").args(args).status().unwrap();
}
