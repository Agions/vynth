//! Date/time utilities

use chrono::{DateTime, Utc};

/// Parse an RFC 3339 datetime string, returning `DateTime::UNIX_EPOCH` (in UTC) on failure.
pub fn parse_rfc3339_or_default(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_rfc3339() {
        let dt = parse_rfc3339_or_default("2024-01-15T10:30:00+00:00");
        assert_eq!(dt.to_rfc3339(), "2024-01-15T10:30:00+00:00");
    }

    #[test]
    fn valid_rfc3339_with_offset() {
        let dt = parse_rfc3339_or_default("2024-06-01T12:00:00+08:00");
        // Should be converted to UTC
        assert_eq!(dt.to_rfc3339(), "2024-06-01T04:00:00+00:00");
    }

    #[test]
    fn invalid_string_falls_back_to_epoch() {
        let dt = parse_rfc3339_or_default("not-a-date");
        assert_eq!(dt, DateTime::UNIX_EPOCH);
    }

    #[test]
    fn empty_string_falls_back_to_epoch() {
        let dt = parse_rfc3339_or_default("");
        assert_eq!(dt, DateTime::UNIX_EPOCH);
    }
}
