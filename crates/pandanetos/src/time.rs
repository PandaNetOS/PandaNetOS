//! 时间工具
//!
//! 提供统一的时间格式转换，遵循 [`docs/standards/data-format.md`] 标准：
//! 所有时间使用 RFC3339 格式、UTC 时区。

use chrono::{DateTime, Utc};

/// 当前时间的 RFC3339 UTC 字符串（格式如 `2026-08-27T10:30:00Z`）
pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// 当前毫秒级时间戳（Unix 毫秒）
pub fn now_millis() -> i64 {
    Utc::now().timestamp_millis()
}

/// 将 RFC3339 字符串解析为 UTC DateTime
///
/// 解析失败时返回 `None`。
pub fn parse_rfc3339(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// 将 DateTime 格式化为 RFC3339 UTC 字符串
pub fn format_rfc3339(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn now_rfc3339_is_utc() {
        let s = now_rfc3339();
        // 必须以 Z 结尾（UTC 标志）
        assert!(s.ends_with('Z'), "RFC3339 UTC 应以 Z 结尾: {s}");
        // 应能解析回去
        let dt = parse_rfc3339(&s);
        assert!(dt.is_some(), "生成的字符串应可被解析: {s}");
    }

    #[test]
    fn parse_accepts_utc_and_offset() {
        let utc = parse_rfc3339("2026-08-27T10:30:00Z");
        assert!(utc.is_some());

        // 带时区偏移的也应能解析并转成 UTC
        let offset = parse_rfc3339("2026-08-27T18:30:00+08:00");
        assert!(offset.is_some());
        assert_eq!(offset.unwrap().hour(), 10, "+08:00 应转换为 UTC 10:30");
    }

    #[test]
    fn parse_invalid_returns_none() {
        assert!(parse_rfc3339("2026-08-27 10:30:00").is_none());
        assert!(parse_rfc3339("not-a-time").is_none());
    }

    #[test]
    fn format_is_round_trip() {
        let original = "2026-08-27T10:30:00Z";
        let dt = parse_rfc3339(original).unwrap();
        assert_eq!(format_rfc3339(dt), original);
    }
}
