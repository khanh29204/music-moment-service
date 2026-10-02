const FRAME_DURATION_MS: f64 = 1152.0 / 44.1;
const FRAME_SIZE_BYTES: f64 = 208.97959;
const XING_FRAME_OFFSET: i64 = 1;

#[derive(Debug, PartialEq)]
pub struct AudioByteRange {
    pub start_byte: i64,
    pub end_byte: i64,
    pub content_length: i64,
    pub actual_start_ms: i64,
    pub actual_end_ms: i64,
}

pub fn calculate_audio_byte_range(start_ms: f64, end_ms: f64) -> AudioByteRange {
    let start_frame = (start_ms / FRAME_DURATION_MS).floor().max(0.0) as i64;
    let end_frame = (end_ms / FRAME_DURATION_MS)
        .ceil()
        .max((start_frame + 1) as f64) as i64;
    let start_byte = (((start_frame + XING_FRAME_OFFSET) as f64) * FRAME_SIZE_BYTES).floor() as i64;
    let end_byte = (((end_frame + XING_FRAME_OFFSET) as f64) * FRAME_SIZE_BYTES).floor() as i64 - 1;
    AudioByteRange {
        start_byte,
        end_byte,
        content_length: end_byte - start_byte + 1,
        actual_start_ms: (start_frame as f64 * FRAME_DURATION_MS).round() as i64,
        actual_end_ms: (end_frame as f64 * FRAME_DURATION_MS).round() as i64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starts_at_zero() {
        assert_eq!(calculate_audio_byte_range(0.0, 30.0).start_byte, 208);
    }
    #[test]
    fn tiny_range_still_contains_one_frame() {
        let r = calculate_audio_byte_range(0.0, 1.0);
        assert!(r.content_length > 0);
    }
    #[test]
    fn large_timestamp_is_monotonic() {
        let r = calculate_audio_byte_range(600_000.0, 600_100.0);
        assert!(r.end_byte > r.start_byte);
    }
}
