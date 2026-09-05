//! BitTorrent 标准类型（PandaNetOS 生态共享）
//!
//! 所有生态项目（PDC、spde、pk）共享的 BT 核心类型。

use std::fmt;
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// L1: Infohash
// ---------------------------------------------------------------------------

/// BitTorrent Infohash（20 字节 SHA1 哈希）
///
/// 支持 hex 和 base32 编码转换。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Infohash(pub [u8; 20]);

impl Infohash {
    /// 零值 infohash
    pub const ZERO: Self = Self([0u8; 20]);

    /// 从字节数组创建
    pub fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// 从切片创建（长度必须为 20）
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 20 {
            return None;
        }
        let mut arr = [0u8; 20];
        arr.copy_from_slice(bytes);
        Some(Self(arr))
    }

    /// 从 hex 字符串解析（40 字符）
    pub fn from_hex(hex: &str) -> Option<Self> {
        if hex.len() != 40 {
            return None;
        }
        let mut bytes = [0u8; 20];
        for i in 0..20 {
            bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
        }
        Some(Self(bytes))
    }

    /// 转换为 hex 字符串（40 字符小写）
    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(40);
        for byte in &self.0 {
            s.push_str(&format!("{:02x}", byte));
        }
        s
    }

    /// 转换为 base32 字符串（32 字符，用于磁力链接）
    pub fn to_base32(&self) -> String {
        const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz234567";
        let mut result = String::with_capacity(32);
        let mut buffer: u64 = 0;
        let mut bits_left = 0;

        for &byte in &self.0 {
            buffer = (buffer << 8) | byte as u64;
            bits_left += 8;
            while bits_left >= 5 {
                bits_left -= 5;
                let idx = ((buffer >> bits_left) & 0x1f) as usize;
                result.push(ALPHABET[idx] as char);
            }
        }
        if bits_left > 0 {
            let idx = ((buffer << (5 - bits_left)) & 0x1f) as usize;
            result.push(ALPHABET[idx] as char);
        }
        result
    }

    /// 获取内部字节数组
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// 是否为零值
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 20]
    }
}

impl fmt::Display for Infohash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 20]> for Infohash {
    fn from(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8]> for Infohash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// L2: PeerInfo
// ---------------------------------------------------------------------------

/// Peer 来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PeerSource {
    Tracker,
    Dht,
    Pex,
    Lpd,
    WebSeed,
    Manual,
}

impl PeerSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            PeerSource::Tracker => "tracker",
            PeerSource::Dht => "dht",
            PeerSource::Pex => "pex",
            PeerSource::Lpd => "lpd",
            PeerSource::WebSeed => "webseed",
            PeerSource::Manual => "manual",
        }
    }
}

impl fmt::Display for PeerSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Peer 信息（标准化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer 地址
    pub addr: SocketAddr,
    /// Peer ID（可选）
    pub peer_id: Option<[u8; 20]>,
    /// 来源
    pub source: PeerSource,
    /// 上传字节数
    pub uploaded: u64,
    /// 下载字节数
    pub downloaded: u64,
    /// 剩余字节数（0 = 做种者）
    pub left: u64,
    /// 最后活跃时间（Unix 时间戳）
    pub last_active: u64,
    /// 连接尝试次数
    pub connection_attempts: u32,
    /// 连接成功次数
    pub connection_successes: u32,
}

impl PeerInfo {
    /// 创建新的 PeerInfo
    pub fn new(addr: SocketAddr, source: PeerSource) -> Self {
        PeerInfo {
            addr,
            peer_id: None,
            source,
            uploaded: 0,
            downloaded: 0,
            left: 0,
            last_active: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            connection_attempts: 0,
            connection_successes: 0,
        }
    }

    /// 是否为做种者（left == 0）
    pub fn is_seeder(&self) -> bool {
        self.left == 0
    }

    /// 是否为 IPv6
    pub fn is_ipv6(&self) -> bool {
        self.addr.is_ipv6()
    }

    /// 连接成功率
    pub fn success_rate(&self) -> f64 {
        if self.connection_attempts == 0 {
            return 0.0;
        }
        self.connection_successes as f64 / self.connection_attempts as f64
    }
}

// ---------------------------------------------------------------------------
// L3: MetadataInfo（种子元数据）
// ---------------------------------------------------------------------------

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// 文件路径
    pub path: String,
    /// 文件大小（字节）
    pub length: u64,
}

/// 种子元数据（info 字典内容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataInfo {
    /// Infohash
    pub infohash: Infohash,
    /// 种子名称
    pub name: String,
    /// 总大小（字节）
    pub total_length: u64,
    /// 分片大小（字节）
    pub piece_length: u64,
    /// 分片数量
    pub piece_count: u32,
    /// 文件列表
    pub files: Vec<FileInfo>,
    /// 创建者
    pub created_by: Option<String>,
    /// 创建时间（Unix 时间戳）
    pub creation_date: Option<u64>,
    /// 评论
    pub comment: Option<String>,
    /// 是否为私有种子
    pub private: bool,
    /// Tracker 列表
    pub trackers: Vec<String>,
    /// 元数据获取时间
    pub fetched_at: u64,
}

impl MetadataInfo {
    /// 创建新的 MetadataInfo
    pub fn new(infohash: Infohash, name: String) -> Self {
        MetadataInfo {
            infohash,
            name,
            total_length: 0,
            piece_length: 0,
            piece_count: 0,
            files: vec![],
            created_by: None,
            creation_date: None,
            comment: None,
            private: false,
            trackers: vec![],
            fetched_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// 生成磁力链接
    pub fn to_magnet_link(&self) -> String {
        let mut link = format!("magnet:?xt=urn:btih:{}", self.infohash.to_hex());
        if !self.name.is_empty() {
            link.push_str(&format!("&dn={}", url_encode(&self.name)));
        }
        for tracker in &self.trackers {
            link.push_str(&format!("&tr={}", url_encode(tracker)));
        }
        link
    }

    /// 格式化大小
    pub fn formatted_size(&self) -> String {
        crate::utils::format_bytes(self.total_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_infohash_hex_roundtrip() {
        let bytes = [1u8; 20];
        let ih = Infohash::new(bytes);
        let hex = ih.to_hex();
        assert_eq!(hex.len(), 40);
        let parsed = Infohash::from_hex(&hex).unwrap();
        assert_eq!(parsed, ih);
    }

    #[test]
    fn test_infohash_base32() {
        let bytes = [0u8; 20];
        let ih = Infohash::new(bytes);
        let b32 = ih.to_base32();
        assert_eq!(b32.len(), 32);
        // 全零应该是全 'a'
        assert!(b32.chars().all(|c| c == 'a'));
    }

    #[test]
    fn test_infohash_from_slice() {
        let bytes = [5u8; 20];
        let ih = Infohash::from_slice(&bytes).unwrap();
        assert_eq!(ih.as_bytes(), &bytes);

        assert!(Infohash::from_slice(&[1u8; 10]).is_none());
    }

    #[test]
    fn test_infohash_is_zero() {
        assert!(Infohash::ZERO.is_zero());
        assert!(!Infohash::new([1u8; 20]).is_zero());
    }

    #[test]
    fn test_peer_info_seeder() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6881);
        let mut peer = PeerInfo::new(addr, PeerSource::Dht);
        assert!(peer.is_seeder());

        peer.left = 100;
        assert!(!peer.is_seeder());
    }

    #[test]
    fn test_peer_info_success_rate() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6881);
        let mut peer = PeerInfo::new(addr, PeerSource::Tracker);
        assert_eq!(peer.success_rate(), 0.0);

        peer.connection_attempts = 10;
        peer.connection_successes = 7;
        assert!((peer.success_rate() - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_metadata_magnet_link() {
        let ih = Infohash::new([0xAB; 20]);
        let mut meta = MetadataInfo::new(ih, "test torrent".to_string());
        meta.trackers.push("http://tracker.example.com/announce".to_string());

        let magnet = meta.to_magnet_link();
        assert!(magnet.starts_with("magnet:?xt=urn:btih:"));
        assert!(magnet.contains("dn="));
        assert!(magnet.contains("tr="));
    }

    #[test]
    fn test_peer_source_display() {
        assert_eq!(PeerSource::Tracker.to_string(), "tracker");
        assert_eq!(PeerSource::Dht.to_string(), "dht");
        assert_eq!(PeerSource::Pex.to_string(), "pex");
    }
}

/// 简单的 URL 编码（percent-encoding）
fn url_encode(s: &str) -> String {
    let mut result = String::new();
    for byte in s.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(*byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}
