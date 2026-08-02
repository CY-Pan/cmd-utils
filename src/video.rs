use serde::Deserialize;
use std::{path::Path, process::Command};

#[derive(Debug, Deserialize, PartialEq)]
pub struct MediaInfo {
    #[serde(default)]
    pub streams: Vec<MediaStreamInfo>,
}

fn deserialize_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    value
        .map(|value| value.parse().map_err(serde::de::Error::custom))
        .transpose()
        .map(|value| value.unwrap_or_default())
}

fn deserialize_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    value
        .map(|value| value.parse().map_err(serde::de::Error::custom))
        .transpose()
        .map(|value| value.unwrap_or_default())
}

fn deserialize_fps<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    let Some(value) = value else {
        return Ok(0.0);
    };
    let (numerator, denominator) = value.split_once('/').unwrap_or((&value, "1"));
    let numerator: f64 = numerator.parse().map_err(serde::de::Error::custom)?;
    let denominator: f64 = denominator.parse().map_err(serde::de::Error::custom)?;
    if denominator == 0.0 {
        return Ok(0.0);
    }
    Ok(numerator / denominator)
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct MediaStreamInfo {
    pub codec_type: Option<String>,
    #[serde(default)]
    pub codec_name: String,
    pub profile: Option<String>,
    pub level: Option<i32>,
    pub time_base: Option<String>,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    #[serde(default)]
    pub pix_fmt: String,
    #[serde(default, rename = "r_frame_rate", deserialize_with = "deserialize_fps")]
    pub fps: f64,
    #[serde(default, deserialize_with = "deserialize_f64")]
    pub duration: f64,
    pub sample_aspect_ratio: Option<String>,
    pub field_order: Option<String>,
    pub sample_fmt: Option<String>,
    pub sample_rate: Option<String>,
    pub channels: Option<u32>,
    pub channel_layout: Option<String>,
    #[serde(default, deserialize_with = "deserialize_u32")]
    pub bit_rate: u32,
}

#[derive(Debug, PartialEq)]
pub struct StreamDifference {
    pub stream_index: Option<usize>,
    pub field: &'static str,
    pub expected: String,
    pub actual: String,
}

pub fn probe_media(path: impl AsRef<Path>) -> MediaInfo {
    let path = path.as_ref();
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=codec_type,codec_name,profile,level,time_base,width,height,pix_fmt,r_frame_rate,duration,sample_aspect_ratio,field_order,sample_fmt,sample_rate,channels,channel_layout,bit_rate",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .unwrap_or_else(|error| panic!("failed to run ffprobe for <{}>: {error}", path.display()));

    assert!(
        output.status.success(),
        "ffprobe failed for <{}>: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr).trim()
    );

    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "failed to parse ffprobe output for <{}>: {error}",
            path.display()
        )
    })
}

fn option_text<T: ToString>(value: &Option<T>) -> String {
    value
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| "<missing>".to_string())
}

pub fn concat_differences(reference: &MediaInfo, candidate: &MediaInfo) -> Vec<StreamDifference> {
    let mut differences = Vec::new();

    if reference.streams.len() != candidate.streams.len() {
        differences.push(StreamDifference {
            stream_index: None,
            field: "stream_count",
            expected: reference.streams.len().to_string(),
            actual: candidate.streams.len().to_string(),
        });
    }

    for (stream_index, (expected, actual)) in
        reference.streams.iter().zip(&candidate.streams).enumerate()
    {
        macro_rules! compare_field {
            ($field:ident) => {
                if expected.$field != actual.$field {
                    differences.push(StreamDifference {
                        stream_index: Some(stream_index),
                        field: stringify!($field),
                        expected: expected.$field.to_string(),
                        actual: actual.$field.to_string(),
                    });
                }
            };
        }

        macro_rules! compare_optional_field {
            ($field:ident) => {
                if expected.$field != actual.$field {
                    differences.push(StreamDifference {
                        stream_index: Some(stream_index),
                        field: stringify!($field),
                        expected: option_text(&expected.$field),
                        actual: option_text(&actual.$field),
                    });
                }
            };
        }

        compare_optional_field!(codec_type);
        compare_field!(codec_name);
        compare_optional_field!(profile);
        compare_optional_field!(level);
        compare_optional_field!(time_base);
        compare_field!(width);
        compare_field!(height);
        compare_field!(pix_fmt);
        compare_field!(fps);
        compare_optional_field!(sample_aspect_ratio);
        compare_optional_field!(field_order);
        compare_optional_field!(sample_fmt);
        compare_optional_field!(sample_rate);
        compare_optional_field!(channels);
        compare_optional_field!(channel_layout);
    }

    differences
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

pub fn get_video_stream(media_info: &MediaInfo) -> Option<&MediaStreamInfo> {
    media_info
        .streams
        .iter()
        .find(|stream| stream.codec_type.as_deref() == Some("video"))
}

pub fn get_audio_stream(media_info: &MediaInfo) -> Option<&MediaStreamInfo> {
    media_info
        .streams
        .iter()
        .find(|stream| stream.codec_type.as_deref() == Some("audio"))
}

pub fn calculate_target_bitrate(media_info: &MediaInfo) -> u32 {
    let video_stream = get_video_stream(media_info).unwrap();
    let width = video_stream.width;
    let height = video_stream.height;
    let fps = video_stream.fps;
    assert!(
        width > 0,
        "cannot calculate target bitrate: video width is missing"
    );
    assert!(
        height > 0,
        "cannot calculate target bitrate: video height is missing"
    );
    assert!(
        fps > 0.0,
        "cannot calculate target bitrate: video frame rate is missing"
    );

    ((2e6 * fps) as u64 * (u64::from(width) * u64::from(height)) / (720 * 1280 * 24)) as u32
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_media_accepts_string_paths() {
        let probe = |path: &str| probe_media(path);
        let _ = probe;
    }

    fn stream(codec_type: &str) -> MediaStreamInfo {
        MediaStreamInfo {
            codec_type: Some(codec_type.to_string()),
            codec_name: String::new(),
            profile: None,
            level: None,
            time_base: None,
            width: 0,
            height: 0,
            pix_fmt: String::new(),
            fps: 0.0,
            duration: 0.0,
            sample_aspect_ratio: None,
            field_order: None,
            sample_fmt: None,
            sample_rate: None,
            channels: None,
            channel_layout: None,
            bit_rate: 0,
        }
    }

    #[test]
    fn get_video_stream_returns_first_match() {
        let mut first = stream("video");
        first.codec_name = "h264".to_string();
        let mut second = stream("video");
        second.codec_name = "hevc".to_string();
        let media_info = MediaInfo {
            streams: vec![stream("audio"), first, second],
        };

        assert_eq!(get_video_stream(&media_info).unwrap().codec_name, "h264");
    }

    #[test]
    fn get_audio_stream_returns_first_match() {
        let mut first = stream("audio");
        first.codec_name = "aac".to_string();
        let mut second = stream("audio");
        second.codec_name = "opus".to_string();
        let media_info = MediaInfo {
            streams: vec![stream("video"), first, second],
        };

        assert_eq!(get_audio_stream(&media_info).unwrap().codec_name, "aac");
    }

    #[test]
    fn concat_differences_accepts_identical_streams() {
        let reference = MediaInfo {
            streams: vec![stream("video")],
        };
        let candidate = MediaInfo {
            streams: vec![stream("video")],
        };

        assert!(concat_differences(&reference, &candidate).is_empty());
    }

    #[test]
    fn concat_differences_reports_stream_count_and_codec_changes() {
        let mut reference_video = stream("video");
        reference_video.codec_name = "h264".to_string();
        let mut candidate_video = stream("video");
        candidate_video.codec_name = "hevc".to_string();
        let reference = MediaInfo {
            streams: vec![reference_video, stream("audio")],
        };
        let candidate = MediaInfo {
            streams: vec![candidate_video],
        };

        let differences = concat_differences(&reference, &candidate);

        assert!(differences.iter().any(|difference| {
            difference.stream_index.is_none()
                && difference.field == "stream_count"
                && difference.expected == "2"
                && difference.actual == "1"
        }));
        assert!(differences.iter().any(|difference| {
            difference.stream_index == Some(0)
                && difference.field == "codec_name"
                && difference.expected == "h264"
                && difference.actual == "hevc"
        }));
    }

    #[test]
    fn media_info_deserializes_ffprobe_streams() {
        let info: MediaInfo = serde_json::from_str(
            r#"{"streams":[{"codec_type":"video","codec_name":"h264","width":1920,"height":1080,"pix_fmt":"yuv420p","r_frame_rate":"30/1","duration":"12.5","bit_rate":"1500000"}]}"#,
        )
        .unwrap();

        assert_eq!(info.streams.len(), 1);
        assert_eq!(info.streams[0].codec_type.as_deref(), Some("video"));
        assert_eq!(info.streams[0].codec_name, "h264");
        assert_eq!(info.streams[0].width, 1920);
        assert_eq!(info.streams[0].height, 1080);
        assert_eq!(info.streams[0].fps, 30.0);
        assert_eq!(info.streams[0].duration, 12.5);
        assert_eq!(info.streams[0].bit_rate, 1_500_000);
        assert_eq!(info.streams[0].pix_fmt, "yuv420p");
    }

    #[test]
    fn media_info_exposes_parsed_fps_and_duration() {
        let info: MediaInfo = serde_json::from_str(
            r#"{"streams":[{"codec_type":"video","r_frame_rate":"30000/1001","duration":"12.5"}]}"#,
        )
        .unwrap();
        let stream = &info.streams[0];

        assert!((stream.fps - 29.97002997002997).abs() < f64::EPSILON);
        assert_eq!(stream.duration, 12.5);
    }

    #[test]
    fn media_info_treats_undefined_fps_as_zero() {
        let info: MediaInfo =
            serde_json::from_str(r#"{"streams":[{"codec_type":"audio","r_frame_rate":"0/0"}]}"#)
                .unwrap();

        assert_eq!(info.streams[0].fps, 0.0);
    }

    #[test]
    fn media_info_defaults_missing_convenience_fields() {
        let info: MediaInfo =
            serde_json::from_str(r#"{"streams":[{"codec_type":"audio"}]}"#).unwrap();

        let stream = &info.streams[0];
        assert_eq!(stream.codec_name, "");
        assert_eq!(stream.width, 0);
        assert_eq!(stream.height, 0);
        assert_eq!(stream.fps, 0.0);
        assert_eq!(stream.duration, 0.0);
        assert_eq!(stream.bit_rate, 0);
        assert_eq!(stream.pix_fmt, "");
    }

    #[test]
    fn target_bitrate_uses_first_video_stream() {
        let mut video = stream("video");
        video.width = 1920;
        video.height = 1080;
        video.fps = 30.0;
        let media_info = MediaInfo {
            streams: vec![stream("audio"), video],
        };

        assert_eq!(calculate_target_bitrate(&media_info), 5_625_000);
    }
}
