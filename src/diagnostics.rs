pub fn frame_ms(delta_seconds: f32) -> f32 {
    if delta_seconds.is_finite() {
        delta_seconds.max(0.0) * 1000.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_ms_never_emits_nan_for_zero_or_invalid_delta() {
        assert_eq!(frame_ms(0.0), 0.0);
        assert_eq!(frame_ms(f32::NAN), 0.0);
        assert_eq!(frame_ms(f32::NEG_INFINITY), 0.0);
        assert!(frame_ms(1.0 / 60.0).is_finite());
    }
}
