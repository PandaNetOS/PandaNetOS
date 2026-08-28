//! 通用工具函数
//!
//! 提供生态内通用的字节格式化、ID 校验等工具。
//! 遵循 [`docs/standards/data-format.md`] 数据格式标准。

use uuid::Uuid;

/// 将字节数格式化为人类可读的容量字符串（B/KB/MB/GB/TB）
///
/// 遵循数据格式标准：以 1024 为进制。
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}

/// 解析人类可读容量字符串为字节数
///
/// 支持格式：`512`、`1KB`、`2.5MB`、`1GB`、`1TB`（大小写不敏感）。
/// 解析失败返回 `None`。
pub fn parse_bytes(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (num_part, unit) = split_unit(s);
    let value: f64 = num_part.parse().ok()?;
    if value < 0.0 {
        return None;
    }
    let multiplier = match unit.to_ascii_uppercase().as_str() {
        "" | "B" => 1u64,
        "KB" | "K" => 1024,
        "MB" | "M" => 1024 * 1024,
        "GB" | "G" => 1024 * 1024 * 1024,
        "TB" | "T" => 1024u64.pow(4),
        _ => return None,
    };
    Some((value * multiplier as f64) as u64)
}

/// 拆分数字与单位
fn split_unit(s: &str) -> (&str, &str) {
    let split = s.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(s.len());
    (&s[..split], &s[split..])
}

/// 校验字符串是否为合法 UUID
pub fn is_valid_uuid(s: &str) -> bool {
    Uuid::parse_str(s).is_ok()
}

/// 生成一个 UUID v4 字符串
pub fn new_uuid() -> String {
    Uuid::new_v4().to_string()
}

/// 将字节数格式化为速度字符串（B/s、KB/s、MB/s…）
pub fn format_speed(bps: u64) -> String {
    format!("{}/s", format_bytes(bps))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_bytes_basic() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn parse_bytes_round_trip() {
        assert_eq!(parse_bytes("0"), Some(0));
        assert_eq!(parse_bytes("1KB"), Some(1024));
        assert_eq!(parse_bytes("1.5MB"), Some((1.5 * 1024.0 * 1024.0) as u64));
        assert_eq!(parse_bytes("1gb"), Some(1024 * 1024 * 1024));
        // 非法输入
        assert_eq!(parse_bytes(""), None);
        assert_eq!(parse_bytes("abc"), None);
        assert_eq!(parse_bytes("-1"), None);
    }

    #[test]
    fn uuid_helpers() {
        let id = new_uuid();
        assert!(is_valid_uuid(&id));
        assert!(!is_valid_uuid("not-a-uuid"));
        assert!(!is_valid_uuid(""));
    }

    #[test]
    fn format_speed_works() {
        assert_eq!(format_speed(0), "0 B/s");
        assert_eq!(format_speed(1024), "1.00 KB/s");
    }
}
