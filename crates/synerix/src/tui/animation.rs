//! Shared TUI animation helpers.
//!
//! Provides frame-driven animation primitives: spinners, dots, easing functions,
//! fade transitions, and pulse effects — all centralized so widgets share one
//! implementation instead of each redefining their own.

// ── Easing Functions ─────────────────────────────────────────────────────────

/// Easing function type: maps a linear progress `t ∈ [0, 1]` to eased progress.
pub type EasingFn = fn(f64) -> f64;

/// Linear (no easing) — constant speed.
pub fn linear(t: f64) -> f64 {
    t
}

/// Ease-in-out cubic — slow start, fast middle, slow end.
pub fn ease_in_out_cubic(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powf(3.0) / 2.0
    }
}

/// Ease-out cubic — fast start, slow deceleration.
pub fn ease_out_cubic(t: f64) -> f64 {
    1.0 - (1.0 - t).powf(3.0)
}

/// Ease-in-out quad — smoother than cubic, good for panel transitions.
pub fn ease_in_out_quad(t: f64) -> f64 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powf(2.0) / 2.0
    }
}

/// Ease-out quad — snappy deceleration for fade-in.
pub fn ease_out_quad(t: f64) -> f64 {
    1.0 - (1.0 - t).powf(2.0)
}

/// Smooth pulse — sine wave between 0.0 and 1.0 over the full period.
///
/// Used for "active" indicators that gently pulse rather than blink.
pub fn pulse_sine(frame: u64, period: u64) -> f64 {
    (2.0 * std::f64::consts::PI * frame as f64 / period as f64).sin().abs()
}

// ── Frame-based Animations ────────────────────────────────────────────────────

/// Cycle a braille spinner frame by animation frame.
///
/// Returns one of: `⣾⣽⣻⢿⡿⣟⣯⣷` (8 frames).
/// Used for "thinking" / "agent working" indicators.
pub fn spinner_frame(frame: u64) -> &'static str {
    const SPINNER: [&str; 8] = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
    SPINNER[(frame as usize) % 8]
}

/// Cycle an ellipsis (``, `.`, `..`, `...`) by animation frame.
///
/// Used for "thinking…" / "running tool…" indicators. The empty first frame
/// keeps the indicator from looking static.
pub fn animated_dots(frame: u64) -> &'static str {
    match frame % 4 {
        0 => "",
        1 => ".",
        2 => "..",
        _ => "...",
    }
}

/// Compute a fade-in alpha value in [0.0, 1.0] for the given frame.
///
/// `duration` is the number of frames the fade should last.
/// Returns `0.0` when `frame >= duration` (fully faded in).
pub fn fade_in_alpha(frame: u64, duration: u64, easing: EasingFn) -> f64 {
    if duration == 0 {
        return 1.0;
    }
    let t = (frame as f64 / duration as f64).min(1.0);
    1.0 - easing(1.0 - t)
}

/// Compute a fade-out alpha value in [0.0, 1.0] for the given frame.
///
/// `duration` is the number of frames the fade should last.
/// Returns `0.0` when `frame >= duration` (fully faded out).
pub fn fade_out_alpha(frame: u64, duration: u64, easing: EasingFn) -> f64 {
    if duration == 0 {
        return 0.0;
    }
    let t = (frame as f64 / duration as f64).min(1.0);
    easing(1.0 - t)
}

/// Compute a smooth blink alpha value in [0.0, 1.0] for the given frame.
///
/// Uses a smooth sine-wave blink — natural, no harsh edges.
/// `period` controls the blink speed (higher = slower).
pub fn blink_alpha(frame: u64, period: u64) -> f64 {
    if period == 0 {
        return 1.0;
    }
    ((2.0 * std::f64::consts::PI * frame as f64 / period as f64).sin() + 1.0) / 2.0
}

// ── Color Interpolation ───────────────────────────────────────────────────────

/// Interpolate between two RGBA colors by a factor `t ∈ [0, 1]`.
///
/// Uses linear RGB interpolation for each channel. Alpha is also interpolated.
pub fn lerp_color(c1: (u8, u8, u8, f64), c2: (u8, u8, u8, f64), t: f64) -> (u8, u8, u8, f64) {
    let t = t.clamp(0.0, 1.0);
    (
        (c1.0 as f64 + t * (c2.0 as f64 - c1.0 as f64)).round() as u8,
        (c1.1 as f64 + t * (c2.1 as f64 - c1.1 as f64)).round() as u8,
        (c1.2 as f64 + t * (c2.2 as f64 - c1.2 as f64)).round() as u8,
        c1.3 + t * (c2.3 - c1.3),
    )
}

/// Convert a color tuple to a ratatui `Color`.
pub fn color_to_ratatui((r, g, b, _): (u8, u8, u8, f64)) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(r, g, b)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animated_dots_cycles_through_four_frames() {
        assert_eq!(animated_dots(0), "");
        assert_eq!(animated_dots(1), ".");
        assert_eq!(animated_dots(2), "..");
        assert_eq!(animated_dots(3), "...");
        assert_eq!(animated_dots(4), "");
    }

    #[test]
    fn spinner_frame_cycles_eight_frames() {
        assert_eq!(spinner_frame(0), "⣾");
        assert_eq!(spinner_frame(7), "⣷");
        assert_eq!(spinner_frame(8), "⣾");
    }

    #[test]
    fn fade_in_alpha_ramps_up() {
        let v0 = fade_in_alpha(0, 10, linear);
        let v5 = fade_in_alpha(5, 10, linear);
        let v10 = fade_in_alpha(10, 10, linear);
        assert!((v0 - 0.0).abs() < 0.01);
        assert!((v5 - 0.5).abs() < 0.01);
        assert!((v10 - 1.0).abs() < 0.01);
    }

    #[test]
    fn fade_out_alpha_ramps_down() {
        let v0 = fade_out_alpha(0, 10, linear);
        let v5 = fade_out_alpha(5, 10, linear);
        let v10 = fade_out_alpha(10, 10, linear);
        assert!((v0 - 1.0).abs() < 0.01);
        assert!((v5 - 0.5).abs() < 0.01);
        assert!((v10 - 0.0).abs() < 0.01);
    }

    #[test]
    fn blink_alpha_produces_sine_wave() {
        let v0 = blink_alpha(0, 100);
        let v25 = blink_alpha(25, 100);
        let v75 = blink_alpha(75, 100);
        // Sine wave oscillates between 0.0 and 1.0
        assert!((v0 - 0.5).abs() < 0.05);
        assert!((v25 - 1.0).abs() < 0.05);   // peak at 1/4 period
        assert!((v75 - 0.0).abs() < 0.05);   // trough at 3/4 period
    }

    #[test]
    fn pulse_sine_bounds() {
        for frame in 0..200 {
            let v = pulse_sine(frame, 60);
            assert!(v >= 0.0 && v <= 1.0, "pulse out of bounds at frame {}: {}", frame, v);
        }
    }

    #[test]
    fn ease_in_out_cubic_symmetry() {
        // Should be 0 at t=0, 0.5 at t=0.5, 1.0 at t=1
        assert!((ease_in_out_cubic(0.0) - 0.0).abs() < 0.01);
        assert!((ease_in_out_cubic(0.5) - 0.5).abs() < 0.01);
        assert!((ease_in_out_cubic(1.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn lerp_color_midpoint() {
        let c = lerp_color((0, 0, 0, 1.0), (255, 255, 255, 0.0), 0.5);
        assert!(((c.0 as i16) - 128).abs() <= 1);
        assert!(((c.1 as i16) - 128).abs() <= 1);
        assert!(((c.2 as i16) - 128).abs() <= 1);
        assert!((c.3 - 0.5).abs() < 0.01);
    }
}
