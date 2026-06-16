/// Shared test utilities for RTCO filter tests.
///
/// Provides:
/// - `count_tokens(text)` — whitespace-token count for savings calculation
/// - `assert_savings(input, output, min_savings)` — asserts token savings >= threshold

/// Count tokens by splitting on whitespace (fast, deterministic).
pub fn count_tokens(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Assert that the filtered output achieves at least `min_savings` percent
/// token savings compared to the raw input.
///
/// # Panics
///
/// Panics when the computed savings percentage is below `min_savings`.
pub fn assert_savings(input: &str, output: &str, min_savings: f64) {
    let input_tokens = count_tokens(input);
    let output_tokens = count_tokens(output);

    // Avoid division by zero when input has no tokens
    if input_tokens == 0 {
        assert!(
            output_tokens == 0,
            "empty input should yield empty output (got {} tokens)",
            output_tokens
        );
        return;
    }

    let savings = 100.0 - (output_tokens as f64 / input_tokens as f64 * 100.0);

    assert!(
        savings >= min_savings - 0.005, // f64 epsilon tolerance
        "Expected ≥{:.0}% token savings, got {:.1}% (input: {} tokens, output: {} tokens)",
        min_savings,
        savings,
        input_tokens,
        output_tokens,
    );
}
