//! 圧縮アルゴリズム
//!
//! データ転送効率を向上させるための圧縮機能を提供します。

use std::io::{self, Read, Write};
use flate2::read::{GzDecoder, ZlibDecoder, DeflateDecoder};
use flate2::write::{GzEncoder, ZlibEncoder, DeflateEncoder};
use flate2::Compression;
use thiserror::Error;

/// 圧縮方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    /// 無圧縮
    None,
    /// Gzip
    Gzip,
    /// Zlib
    Zlib,
    /// Deflate
    Deflate,
    /// LZ4
    LZ4,
    /// Zstandard
    Zstd,
}

impl CompressionAlgorithm {
    /// 文字列から圧縮アルゴリズムを解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(CompressionAlgorithm::None),
            "gzip" => Some(CompressionAlgorithm::Gzip),
            "zlib" => Some(CompressionAlgorithm::Zlib),
            "deflate" => Some(CompressionAlgorithm::Deflate),
            "lz4" => Some(CompressionAlgorithm::LZ4),
            "zstd" => Some(CompressionAlgorithm::Zstd),
            _ => None,
        }
    }
    
    /// 圧縮アルゴリズムを文字列に変換
    pub fn as_str(&self) -> &'static str {
        match self {
            CompressionAlgorithm::None => "none",
            CompressionAlgorithm::Gzip => "gzip",
            CompressionAlgorithm::Zlib => "zlib",
            CompressionAlgorithm::Deflate => "deflate",
            CompressionAlgorithm::LZ4 => "lz4",
            CompressionAlgorithm::Zstd => "zstd",
        }
    }
    
    /// MIME タイプを取得
    pub fn content_encoding(&self) -> Option<&'static str> {
        match self {
            CompressionAlgorithm::None => None,
            CompressionAlgorithm::Gzip => Some("gzip"),
            CompressionAlgorithm::Zlib => None, // コンテンツエンコーディングに標準の指定がない
            CompressionAlgorithm::Deflate => Some("deflate"),
            CompressionAlgorithm::LZ4 => None, // 標準的なMIMEタイプがない
            CompressionAlgorithm::Zstd => Some("zstd"),
        }
    }
}

/// 圧縮エラー
#[derive(Error, Debug)]
pub enum CompressionError {
    /// I/O エラー
    #[error("圧縮・解凍中にI/Oエラーが発生しました: {0}")]
    IoError(#[from] io::Error),
    
    /// サポートされていない圧縮方式
    #[error("サポートされていない圧縮方式です: {0}")]
    UnsupportedAlgorithm(String),
    
    /// その他のエラー
    #[error("圧縮・解凍中にエラーが発生しました: {0}")]
    Other(String),
}

/// 圧縮レベル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// 無圧縮
    None,
    /// 最速（圧縮率より速度優先）
    Fastest,
    /// 速い
    Fast,
    /// デフォルト
    Default,
    /// 良い圧縮率
    Good,
    /// 最良（速度より圧縮率優先）
    Best,
}

impl CompressionLevel {
    /// flate2 圧縮レベルに変換
    fn to_flate2_level(&self) -> Compression {
        match self {
            CompressionLevel::None => Compression::none(),
            CompressionLevel::Fastest => Compression::fast(),
            CompressionLevel::Fast => Compression::new(3),
            CompressionLevel::Default => Compression::default(),
            CompressionLevel::Good => Compression::new(7),
            CompressionLevel::Best => Compression::best(),
        }
    }
    
    /// LZ4 圧縮レベルに変換
    #[cfg(feature = "lz4")]
    fn to_lz4_level(&self) -> i32 {
        match self {
            CompressionLevel::None => 0,
            CompressionLevel::Fastest => 1,
            CompressionLevel::Fast => 3,
            CompressionLevel::Default => 6,
            CompressionLevel::Good => 9,
            CompressionLevel::Best => 12,
        }
    }
    
    /// Zstandard 圧縮レベルに変換
    #[cfg(feature = "zstd")]
    fn to_zstd_level(&self) -> i32 {
        match self {
            CompressionLevel::None => 1,
            CompressionLevel::Fastest => 1,
            CompressionLevel::Fast => 3,
            CompressionLevel::Default => 5,
            CompressionLevel::Good => 15,
            CompressionLevel::Best => 22,
        }
    }
}

/// データを圧縮
pub fn compress(
    data: &[u8],
    algorithm: CompressionAlgorithm,
    level: CompressionLevel,
) -> Result<Vec<u8>, CompressionError> {
    match algorithm {
        CompressionAlgorithm::None => Ok(data.to_vec()),
        
        CompressionAlgorithm::Gzip => {
            let mut encoder = GzEncoder::new(Vec::new(), level.to_flate2_level());
            encoder.write_all(data)?;
            Ok(encoder.finish()?)
        },
        
        CompressionAlgorithm::Zlib => {
            let mut encoder = ZlibEncoder::new(Vec::new(), level.to_flate2_level());
            encoder.write_all(data)?;
            Ok(encoder.finish()?)
        },
        
        CompressionAlgorithm::Deflate => {
            let mut encoder = DeflateEncoder::new(Vec::new(), level.to_flate2_level());
            encoder.write_all(data)?;
            Ok(encoder.finish()?)
        },
        
        CompressionAlgorithm::LZ4 => {
            #[cfg(feature = "lz4")]
            {
                let mut compressed = Vec::new();
                let preferences = lz4::block::CompressionOptions::new()
                    .compression_level(level.to_lz4_level())
                    .favor_dec_speed(level == CompressionLevel::Fastest);
                
                lz4::block::compress_to_vec(data, &mut compressed, Some(preferences))
                    .map_err(|e| CompressionError::Other(format!("LZ4 圧縮エラー: {}", e)))?;
                
                Ok(compressed)
            }
            
            #[cfg(not(feature = "lz4"))]
            {
                Err(CompressionError::UnsupportedAlgorithm("LZ4 圧縮はこのビルドではサポートされていません".to_string()))
            }
        },
        
        CompressionAlgorithm::Zstd => {
            #[cfg(feature = "zstd")]
            {
                let level = level.to_zstd_level();
                zstd::encode_all(std::io::Cursor::new(data), level)
                    .map_err(|e| CompressionError::Other(format!("Zstandard 圧縮エラー: {}", e)))
            }
            
            #[cfg(not(feature = "zstd"))]
            {
                Err(CompressionError::UnsupportedAlgorithm("Zstandard 圧縮はこのビルドではサポートされていません".to_string()))
            }
        },
    }
}

/// データを解凍
pub fn decompress(
    data: &[u8],
    algorithm: CompressionAlgorithm,
) -> Result<Vec<u8>, CompressionError> {
    match algorithm {
        CompressionAlgorithm::None => Ok(data.to_vec()),
        
        CompressionAlgorithm::Gzip => {
            let mut decoder = GzDecoder::new(data);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            Ok(decompressed)
        },
        
        CompressionAlgorithm::Zlib => {
            let mut decoder = ZlibDecoder::new(data);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            Ok(decompressed)
        },
        
        CompressionAlgorithm::Deflate => {
            let mut decoder = DeflateDecoder::new(data);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            Ok(decompressed)
        },
        
        CompressionAlgorithm::LZ4 => {
            #[cfg(feature = "lz4")]
            {
                let mut decompressed = Vec::new();
                lz4::block::decompress_to_vec(data, &mut decompressed)
                    .map_err(|e| CompressionError::Other(format!("LZ4 解凍エラー: {}", e)))?;
                Ok(decompressed)
            }
            
            #[cfg(not(feature = "lz4"))]
            {
                Err(CompressionError::UnsupportedAlgorithm("LZ4 解凍はこのビルドではサポートされていません".to_string()))
            }
        },
        
        CompressionAlgorithm::Zstd => {
            #[cfg(feature = "zstd")]
            {
                zstd::decode_all(std::io::Cursor::new(data))
                    .map_err(|e| CompressionError::Other(format!("Zstandard 解凍エラー: {}", e)))
            }
            
            #[cfg(not(feature = "zstd"))]
            {
                Err(CompressionError::UnsupportedAlgorithm("Zstandard 解凍はこのビルドではサポートされていません".to_string()))
            }
        },
    }
}

/// 圧縮ストリーム
pub struct CompressionStream {
    inner: Box<dyn Write>,
    algorithm: CompressionAlgorithm,
}

impl CompressionStream {
    /// 新しい圧縮ストリームを作成
    pub fn new<W: Write + 'static>(writer: W, algorithm: CompressionAlgorithm, level: CompressionLevel) -> Result<Self, CompressionError> {
        let inner: Box<dyn Write> = match algorithm {
            CompressionAlgorithm::None => Box::new(writer),
            CompressionAlgorithm::Gzip => Box::new(GzEncoder::new(writer, level.to_flate2_level())),
            CompressionAlgorithm::Zlib => Box::new(ZlibEncoder::new(writer, level.to_flate2_level())),
            CompressionAlgorithm::Deflate => Box::new(DeflateEncoder::new(writer, level.to_flate2_level())),
            
            CompressionAlgorithm::LZ4 | CompressionAlgorithm::Zstd => {
                return Err(CompressionError::UnsupportedAlgorithm(
                    format!("{} ストリーミング圧縮はサポートされていません", algorithm.as_str())
                ));
            }
        };
        
        Ok(Self {
            inner,
            algorithm,
        })
    }
}

impl Write for CompressionStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// 解凍ストリーム
pub struct DecompressionStream {
    inner: Box<dyn Read>,
    algorithm: CompressionAlgorithm,
}

impl DecompressionStream {
    /// 新しい解凍ストリームを作成
    pub fn new<R: Read + 'static>(reader: R, algorithm: CompressionAlgorithm) -> Result<Self, CompressionError> {
        let inner: Box<dyn Read> = match algorithm {
            CompressionAlgorithm::None => Box::new(reader),
            CompressionAlgorithm::Gzip => Box::new(GzDecoder::new(reader)),
            CompressionAlgorithm::Zlib => Box::new(ZlibDecoder::new(reader)),
            CompressionAlgorithm::Deflate => Box::new(DeflateDecoder::new(reader)),
            
            CompressionAlgorithm::LZ4 | CompressionAlgorithm::Zstd => {
                return Err(CompressionError::UnsupportedAlgorithm(
                    format!("{} ストリーミング解凍はサポートされていません", algorithm.as_str())
                ));
            }
        };
        
        Ok(Self {
            inner,
            algorithm,
        })
    }
    
    /// 解凍したデータをすべて読み出す
    pub fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, CompressionError> {
        Ok(self.inner.read_to_end(buf)?)
    }
}

impl Read for DecompressionStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

/// 利用可能な圧縮アルゴリズムを取得
pub fn available_algorithms() -> Vec<CompressionAlgorithm> {
    let algorithms = vec![
        CompressionAlgorithm::None,
        CompressionAlgorithm::Gzip,
        CompressionAlgorithm::Zlib,
        CompressionAlgorithm::Deflate,
    ];
    
    #[cfg(feature = "lz4")]
    {
        let mut algs = algorithms;
        algs.push(CompressionAlgorithm::LZ4);
        algs
    }
    
    #[cfg(not(feature = "lz4"))]
    {
        #[cfg(feature = "zstd")]
        {
            let mut algs = algorithms;
            algs.push(CompressionAlgorithm::Zstd);
            algs
        }
        
        #[cfg(not(feature = "zstd"))]
        {
            algorithms
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compress_decompress() {
        let data = b"This is some test data to compress and decompress";
        
        for algorithm in &[CompressionAlgorithm::Gzip, CompressionAlgorithm::Zlib, CompressionAlgorithm::Deflate] {
            let compressed = compress(data, *algorithm, CompressionLevel::Default)
                .expect(&format!("{} 圧縮に失敗しました", algorithm.as_str()));
            
            let decompressed = decompress(&compressed, *algorithm)
                .expect(&format!("{} 解凍に失敗しました", algorithm.as_str()));
            
            assert_eq!(data.as_ref(), decompressed.as_slice());
        }
    }
    
    #[test]
    fn test_compression_level() {
        let data = vec![0; 1000]; // 0で埋められた1000バイトのデータ
        
        let compressed_best = compress(&data, CompressionAlgorithm::Gzip, CompressionLevel::Best)
            .expect("Gzip 圧縮に失敗しました");
        
        let compressed_fastest = compress(&data, CompressionAlgorithm::Gzip, CompressionLevel::Fastest)
            .expect("Gzip 圧縮に失敗しました");
        
        // 最適な圧縮は、最速の圧縮よりも小さくなるはず
        assert!(compressed_best.len() <= compressed_fastest.len());
    }
}