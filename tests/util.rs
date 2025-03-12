#[cfg(test)]
mod tests {
    use std::i16;

    fn float_to_i16(value: f32) -> i16 {
        (value * (i16::MAX as f32 + 1.0)) as i16
    }

    #[test]
    fn test_float_to_i16_conversion() {
        assert_eq!(
            float_to_i16(-1.0),
            i16::MIN,
            "Expected -1.0 to map to i16::MIN"
        );
        assert_eq!(float_to_i16(0.0), 0, "Expected 0.0 to map to 0");
        assert_eq!(
            float_to_i16(1.0),
            i16::MAX,
            "Expected 1.0 to map to i16::MAX"
        );

        // Additional sanity checks
        assert!(float_to_i16(-0.5) < 0, "Expected -0.5 to be negative");
        assert!(float_to_i16(0.5) > 0, "Expected 0.5 to be positive");
    }
}
