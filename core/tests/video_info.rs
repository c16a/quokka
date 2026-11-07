//! The video module against real files. The expected values are what ffprobe reports for the
//! same fixtures, which were made with ffmpeg in `core/tests/fixtures/video`.

use std::path::{Path, PathBuf};

use player_core::video::{self, Chroma, Codec, Container, FrameRate, VideoFormat};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/video")
        .join(name)
}

/// One fixture's expected format.
struct Expect {
    file: &'static str,
    container: Container,
    codec: Codec,
    width: u32,
    height: u32,
    bit_depth: u8,
    chroma: Chroma,
    /// Frames per second as a fraction, or `None` when the file does not state one.
    fps: Option<(u64, u64)>,
    /// Display aspect ratio, or `None` when the file does not state one.
    aspect: Option<f64>,
}

const WIDE: f64 = 16.0 / 9.0;
const RATE: Option<(u64, u64)> = Some((24000, 1001));

fn expectations() -> Vec<Expect> {
    use Chroma::*;
    use Codec::*;
    use Container::*;
    let base = |file, container, codec, depth, chroma, fps, aspect| Expect {
        file,
        container,
        codec,
        width: 320,
        height: 180,
        bit_depth: depth,
        chroma,
        fps,
        aspect,
    };
    vec![
        base("h264_8_420.mp4", Mp4, H264, 8, Yuv420, RATE, Some(WIDE)),
        base(
            "h264_8_420.mkv",
            Matroska,
            H264,
            8,
            Yuv420,
            RATE,
            Some(WIDE),
        ),
        base("h264_8_420.h264", AnnexB, H264, 8, Yuv420, RATE, Some(WIDE)),
        base("hevc_8_420.mp4", Mp4, Hevc, 8, Yuv420, RATE, Some(WIDE)),
        base(
            "hevc_8_420.mkv",
            Matroska,
            Hevc,
            8,
            Yuv420,
            RATE,
            Some(WIDE),
        ),
        base("hevc_8_420.hevc", AnnexB, Hevc, 8, Yuv420, RATE, Some(WIDE)),
        base("av1_8_420.mp4", Mp4, Av1, 8, Yuv420, RATE, Some(WIDE)),
        base("av1_8_420.mkv", Matroska, Av1, 8, Yuv420, RATE, Some(WIDE)),
        base("av1_8_420.webm", Matroska, Av1, 8, Yuv420, RATE, Some(WIDE)),
        // Raw AV1 states no frame rate; its aspect is the picture's, with square pixels.
        base("av1_8_420.obu", RawObu, Av1, 8, Yuv420, None, Some(WIDE)),
        base("vp9_8_420.webm", Matroska, Vp9, 8, Yuv420, RATE, Some(WIDE)),
        base("vp9_8_420.mkv", Matroska, Vp9, 8, Yuv420, RATE, Some(WIDE)),
        base("h264_10_420.mp4", Mp4, H264, 10, Yuv420, RATE, Some(WIDE)),
        base("hevc_10_420.mp4", Mp4, Hevc, 10, Yuv420, RATE, Some(WIDE)),
        base(
            "hevc_10_420.hevc",
            AnnexB,
            Hevc,
            10,
            Yuv420,
            RATE,
            Some(WIDE),
        ),
        base(
            "av1_10_420.mkv",
            Matroska,
            Av1,
            10,
            Yuv420,
            RATE,
            Some(WIDE),
        ),
        base("av1_10_420.obu", RawObu, Av1, 10, Yuv420, None, Some(WIDE)),
        base(
            "vp9_10_420.webm",
            Matroska,
            Vp9,
            10,
            Yuv420,
            RATE,
            Some(WIDE),
        ),
        base("h264_10_422.mp4", Mp4, H264, 10, Yuv422, RATE, Some(WIDE)),
        base("hevc_10_422.mp4", Mp4, Hevc, 10, Yuv422, RATE, Some(WIDE)),
        base(
            "h264_8_444.mkv",
            Matroska,
            H264,
            8,
            Yuv444,
            RATE,
            Some(WIDE),
        ),
        base("hevc_8_444.mp4", Mp4, Hevc, 8, Yuv444, RATE, Some(WIDE)),
        // Stored 320x240 with a 16:9 display aspect ratio: the container's ratio wins.
        Expect {
            height: 240,
            ..base("h264_8_420_dar.mp4", Mp4, H264, 8, Yuv420, RATE, Some(WIDE))
        },
    ]
}

/// Compare one file's format with its expectation, returning every mismatch.
fn mismatches(expect: &Expect, format: &VideoFormat) -> Vec<String> {
    let mut problems = Vec::new();
    let mut check = |name: &str, ok: bool, got: String, want: String| {
        if !ok {
            problems.push(format!("{name}: got {got}, want {want}"));
        }
    };
    check(
        "container",
        format.container == Some(expect.container),
        format!("{:?}", format.container),
        format!("{:?}", expect.container),
    );
    check(
        "codec",
        format.codec == Some(expect.codec),
        format!("{:?}", format.codec),
        format!("{:?}", expect.codec),
    );
    check(
        "width",
        format.width == Some(expect.width),
        format!("{:?}", format.width),
        expect.width.to_string(),
    );
    check(
        "height",
        format.height == Some(expect.height),
        format!("{:?}", format.height),
        expect.height.to_string(),
    );
    check(
        "bit depth",
        format.bit_depth == Some(expect.bit_depth),
        format!("{:?}", format.bit_depth),
        expect.bit_depth.to_string(),
    );
    check(
        "chroma",
        format.chroma == Some(expect.chroma),
        format!("{:?}", format.chroma.map(Chroma::name)),
        expect.chroma.name().to_owned(),
    );
    let fps_ok = match (format.frame_rate, expect.fps) {
        (None, None) => true,
        (Some(rate), Some((numerator, denominator))) => {
            rate == FrameRate {
                numerator,
                denominator,
            }
        }
        _ => false,
    };
    check(
        "frame rate",
        fps_ok,
        format!("{:?}", format.frame_rate),
        format!("{:?}", expect.fps),
    );
    let aspect_ok = match (format.display_aspect, expect.aspect) {
        (None, None) => true,
        (Some(got), Some(want)) => (got - want).abs() < 0.001,
        _ => false,
    };
    check(
        "aspect",
        aspect_ok,
        format!("{:?}", format.display_aspect),
        format!("{:?}", expect.aspect),
    );
    problems
}

#[test]
fn every_fixture_reports_its_format() {
    let mut failures = Vec::new();
    for expect in expectations() {
        let format = video::inspect(&fixture(expect.file)).expect("the fixture can be read");
        for problem in mismatches(&expect, &format) {
            failures.push(format!("{}: {problem}", expect.file));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn a_file_that_is_not_video_reports_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.txt");
    std::fs::write(&path, b"just some text, not a video").unwrap();
    let format = video::inspect(&path).expect("the file can be read");
    assert_eq!(format, VideoFormat::default());
}
