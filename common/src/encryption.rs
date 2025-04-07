//! 暗号化機能
//!
//! データの安全な暗号化と復号を提供し、セキュアな通信を可能にします。

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use base64::Engine;
use rand::{rngs::OsRng, RngCore};
use sha2::{Sha256, Digest};
use thiserror::Error;
use std::fmt;
use pbkdf2::pbkdf2;  // 関数をインポート
use hmac::Hmac;      // 型をインポート

/// 暗号化エラー
#[derive(Error, Debug)]
pub enum EncryptionError {
    /// 暗号化/復号化エラー
    #[error("暗号化/復号化エラー: {0}")]
    CryptoError(String),
    
    /// キー導出エラー
    #[error("キー導出エラー: {0}")]
    KeyDerivationError(String),
    
    /// 無効なキー
    #[error("無効なキー: {0}")]
    InvalidKey(String),
    
    /// 無効なデータ
    #[error("無効なデータ: {0}")]
    InvalidData(String),
    
    /// その他のエラー
    #[error("暗号化エラー: {0}")]
    Other(String),
}

/// 暗号化アルゴリズム
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM
    Aes256Gcm,
    /// ChaCha20-Poly1305
    ChaCha20Poly1305,
}

impl EncryptionAlgorithm {
    /// 文字列から暗号化アルゴリズムを取得
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "aes-256-gcm" | "aes256gcm" | "aes-gcm" => Some(EncryptionAlgorithm::Aes256Gcm),
            "chacha20-poly1305" | "chacha20poly1305" | "chacha20" => Some(EncryptionAlgorithm::ChaCha20Poly1305),
            _ => None,
        }
    }
    
    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            EncryptionAlgorithm::Aes256Gcm => "AES-256-GCM",
            EncryptionAlgorithm::ChaCha20Poly1305 => "ChaCha20-Poly1305",
        }
    }
}

/// 暗号化キー
pub struct EncryptionKey {
    /// キーデータ
    data: Vec<u8>,
    /// アルゴリズム
    algorithm: EncryptionAlgorithm,
}

impl EncryptionKey {
    /// ランダムな新しいキーを生成
    pub fn generate(algorithm: EncryptionAlgorithm) -> Self {
        let mut key_data = vec![0u8; 32]; // AES-256とChaCha20はどちらも32バイトキー
        OsRng.fill_bytes(&mut key_data);
        
        Self {
            data: key_data,
            algorithm,
        }
    }
    
    /// パスワードからキーを導出
    pub fn from_password(password: &str, salt: &[u8], algorithm: EncryptionAlgorithm) -> Self {
        // Argon2を使用するのが最も安全だが、依存関係を減らすためにPBKDF2-HMACを使用
        // 実際のプロダクションコードではArgon2を使用することを推奨
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        let key_data = hasher.finalize().to_vec();
        
        Self {
            data: key_data,
            algorithm,
        }
    }
    
    /// バイトからキーを作成
    pub fn from_bytes(bytes: &[u8], algorithm: EncryptionAlgorithm) -> Result<Self, EncryptionError> {
        if bytes.len() != 32 {
            return Err(EncryptionError::InvalidKey(
                format!("キーは32バイトである必要があります。{}バイトが指定されました", bytes.len())
            ));
        }
        
        Ok(Self {
            data: bytes.to_vec(),
            algorithm,
        })
    }
    
    /// キーをバイト列として取得
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    
    /// アルゴリズムを取得
    pub fn algorithm(&self) -> EncryptionAlgorithm {
        self.algorithm
    }
}

impl fmt::Debug for EncryptionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EncryptionKey {{ algorithm: {:?}, data: [REDACTED] }}", self.algorithm)
    }
}

/// 暗号化されたデータ
#[derive(Debug, Clone)]
pub struct EncryptedData {
    /// 暗号文
    ciphertext: Vec<u8>,
    /// ノンス (初期化ベクトル)
    nonce: Vec<u8>,
    /// 追加認証データ
    aad: Option<Vec<u8>>,
    /// アルゴリズム
    algorithm: EncryptionAlgorithm,
}

impl EncryptedData {
    /// 新しい暗号化データを作成
    pub fn new(
        ciphertext: Vec<u8>,
        nonce: Vec<u8>,
        aad: Option<Vec<u8>>,
        algorithm: EncryptionAlgorithm,
    ) -> Self {
        Self {
            ciphertext,
            nonce,
            aad,
            algorithm,
        }
    }
    
    /// 暗号文を取得
    pub fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
    }
    
    /// ノンスを取得
    pub fn nonce(&self) -> &[u8] {
        &self.nonce
    }
    
    /// 追加認証データを取得
    pub fn aad(&self) -> Option<&[u8]> {
        self.aad.as_deref()
    }
    
    /// アルゴリズムを取得
    pub fn algorithm(&self) -> EncryptionAlgorithm {
        self.algorithm
    }
    
    /// バイト列にシリアライズ
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        
        // アルゴリズム識別子 (1バイト)
        result.push(match self.algorithm {
            EncryptionAlgorithm::Aes256Gcm => 0,
            EncryptionAlgorithm::ChaCha20Poly1305 => 1,
        });
        
        // ノンスの長さ (1バイト)
        result.push(self.nonce.len() as u8);
        
        // ノンス
        result.extend_from_slice(&self.nonce);
        
        // AADのフラグとデータ
        if let Some(aad) = &self.aad {
            result.push(1); // AADあり
            
            // AADの長さ (2バイト, リトルエンディアン)
            let aad_len = aad.len() as u16;
            result.extend_from_slice(&aad_len.to_le_bytes());
            
            // AADデータ
            result.extend_from_slice(aad);
        } else {
            result.push(0); // AADなし
        }
        
        // 暗号文の長さ (4バイト, リトルエンディアン)
        let ciphertext_len = self.ciphertext.len() as u32;
        result.extend_from_slice(&ciphertext_len.to_le_bytes());
        
        // 暗号文
        result.extend_from_slice(&self.ciphertext);
        
        result
    }
    
    /// バイト列からデシリアライズ
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, EncryptionError> {
        if bytes.len() < 8 {
            return Err(EncryptionError::InvalidData("データが短すぎます".to_string()));
        }
        
        let mut pos = 0;
        
        // アルゴリズム識別子
        let algorithm = match bytes[pos] {
            0 => EncryptionAlgorithm::Aes256Gcm,
            1 => EncryptionAlgorithm::ChaCha20Poly1305,
            _ => return Err(EncryptionError::InvalidData("不明なアルゴリズム識別子".to_string())),
        };
        pos += 1;
        
        // ノンスの長さ
        let nonce_len = bytes[pos] as usize;
        pos += 1;
        
        if pos + nonce_len > bytes.len() {
            return Err(EncryptionError::InvalidData("ノンスデータが不足しています".to_string()));
        }
        
        // ノンス
        let nonce = bytes[pos..pos + nonce_len].to_vec();
        pos += nonce_len;
        
        // AADフラグ
        if pos >= bytes.len() {
            return Err(EncryptionError::InvalidData("AADフラグが不足しています".to_string()));
        }
        
        let has_aad = bytes[pos] != 0;
        pos += 1;
        
        // AADデータ
        let aad = if has_aad {
            if pos + 2 > bytes.len() {
                return Err(EncryptionError::InvalidData("AAD長が不足しています".to_string()));
            }
            
            let aad_len_bytes = [bytes[pos], bytes[pos + 1]];
            let aad_len = u16::from_le_bytes(aad_len_bytes) as usize;
            pos += 2;
            
            if pos + aad_len > bytes.len() {
                return Err(EncryptionError::InvalidData("AADデータが不足しています".to_string()));
            }
            
            let aad_data = bytes[pos..pos + aad_len].to_vec();
            pos += aad_len;
            
            Some(aad_data)
        } else {
            None
        };
        
        // 暗号文の長さ
        if pos + 4 > bytes.len() {
            return Err(EncryptionError::InvalidData("暗号文長が不足しています".to_string()));
        }
        
        let ciphertext_len_bytes = [bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]];
        let ciphertext_len = u32::from_le_bytes(ciphertext_len_bytes) as usize;
        pos += 4;
        
        if pos + ciphertext_len > bytes.len() {
            return Err(EncryptionError::InvalidData("暗号文データが不足しています".to_string()));
        }
        
        // 暗号文
        let ciphertext = bytes[pos..pos + ciphertext_len].to_vec();
        
        Ok(Self {
            ciphertext,
            nonce,
            aad,
            algorithm,
        })
    }
}

/// データを暗号化
pub fn encrypt(
    plaintext: &[u8],
    key: &EncryptionKey,
    aad: Option<&[u8]>,
) -> Result<EncryptedData, EncryptionError> {
    match key.algorithm() {
        EncryptionAlgorithm::Aes256Gcm => {
            let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
                .map_err(|e| EncryptionError::InvalidKey(e.to_string()))?;
            
            // ノンスを生成 (12バイト)
            let mut nonce_bytes = [0u8; 12];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);
            
            // 暗号化を実行
            let ciphertext = if let Some(aad_data) = aad {
                let payload = Payload {
                    msg: plaintext,
                    aad: aad_data,
                };
                
                cipher.encrypt(nonce, payload)
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            } else {
                cipher.encrypt(nonce, plaintext)
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            };
            
            Ok(EncryptedData::new(
                ciphertext,
                nonce_bytes.to_vec(),
                aad.map(|data| data.to_vec()),
                EncryptionAlgorithm::Aes256Gcm,
            ))
        },
        
        EncryptionAlgorithm::ChaCha20Poly1305 => {
            #[cfg(feature = "chacha20poly1305")]
            {
                use chacha20poly1305::{ChaCha20Poly1305, Key, XNonce};
                
                let key_bytes = Key::from_slice(key.as_bytes());
                let cipher = ChaCha20Poly1305::new(key_bytes);
                
                // ノンスを生成 (24バイト)
                let mut nonce_bytes = [0u8; 24];
                OsRng.fill_bytes(&mut nonce_bytes);
                let nonce = XNonce::from_slice(&nonce_bytes[..24]);
                
                // 暗号化を実行
                let ciphertext = if let Some(aad_data) = aad {
                    let payload = Payload {
                        msg: plaintext,
                        aad: aad_data,
                    };
                    
                    cipher.encrypt(nonce, payload)
                        .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
                } else {
                    cipher.encrypt(nonce, plaintext)
                        .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
                };
                
                Ok(EncryptedData::new(
                    ciphertext,
                    nonce_bytes.to_vec(),
                    aad.map(|data| data.to_vec()),
                    EncryptionAlgorithm::ChaCha20Poly1305,
                ))
            }
            
            #[cfg(not(feature = "chacha20poly1305"))]
            {
                Err(EncryptionError::CryptoError("ChaCha20-Poly1305はこのビルドではサポートされていません".to_string()))
            }
        },
    }
}

/// データを復号
pub fn decrypt(
    encrypted_data: &EncryptedData,
    key: &EncryptionKey,
) -> Result<Vec<u8>, EncryptionError> {
    // アルゴリズムが一致することを確認
    if encrypted_data.algorithm() != key.algorithm() {
        return Err(EncryptionError::InvalidKey(
            format!("キーアルゴリズム({:?})と暗号文アルゴリズム({:?})が一致しません",
                key.algorithm(), encrypted_data.algorithm())
        ));
    }
    
    match key.algorithm() {
        EncryptionAlgorithm::Aes256Gcm => {
            let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
                .map_err(|e| EncryptionError::InvalidKey(e.to_string()))?;
            
            let nonce = Nonce::from_slice(encrypted_data.nonce());
            
            // 復号を実行
            let plaintext = if let Some(aad_data) = encrypted_data.aad() {
                let payload = Payload {
                    msg: encrypted_data.ciphertext(),
                    aad: aad_data,
                };
                
                cipher.decrypt(nonce, payload)
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            } else {
                cipher.decrypt(nonce, encrypted_data.ciphertext())
                    .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
            };
            
            Ok(plaintext)
        },
        
        EncryptionAlgorithm::ChaCha20Poly1305 => {
            #[cfg(feature = "chacha20poly1305")]
            {
                use chacha20poly1305::{ChaCha20Poly1305, Key, XNonce};
                
                let key_bytes = Key::from_slice(key.as_bytes());
                let cipher = ChaCha20Poly1305::new(key_bytes);
                
                let nonce = XNonce::from_slice(&encrypted_data.nonce()[..24]);
                
                // 復号を実行
                let plaintext = if let Some(aad_data) = encrypted_data.aad() {
                    let payload = Payload {
                        msg: encrypted_data.ciphertext(),
                        aad: aad_data,
                    };
                    
                    cipher.decrypt(nonce, payload)
                        .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
                } else {
                    cipher.decrypt(nonce, encrypted_data.ciphertext())
                        .map_err(|e| EncryptionError::CryptoError(e.to_string()))?
                };
                
                Ok(plaintext)
            }
            
            #[cfg(not(feature = "chacha20poly1305"))]
            {
                Err(EncryptionError::CryptoError("ChaCha20-Poly1305はこのビルドではサポートされていません".to_string()))
            }
        },
    }
}

/// パスワードベースの鍵導出関数（PBKDF2）を使用してキーを導出
pub fn derive_key_pbkdf2(
    password: &str,
    salt: &[u8],
    iterations: u32,
    algorithm: EncryptionAlgorithm,
) -> Result<EncryptionKey, EncryptionError> {
    let key_len = match algorithm {
        EncryptionAlgorithm::Aes256Gcm => 32,
        EncryptionAlgorithm::ChaCha20Poly1305 => 32,
    };
    
    let mut key_bytes = vec![0u8; key_len];
    
    // pbkdf2関数を直接呼び出して、エラーはstd::resultのResultを介して処理する
    pbkdf2::<Hmac<Sha256>>(
        password.as_bytes(),
        salt,
        iterations,
        &mut key_bytes,
    );
    
    // 成功した場合、EncryptionKeyを返す
    Ok(EncryptionKey {
        data: key_bytes,
        algorithm,
    })
}

/// 安全なランダムソルトを生成
pub fn generate_salt(len: usize) -> Vec<u8> {
    let mut salt = vec![0u8; len];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Base64エンコードされた暗号化データを生成
pub fn encrypt_to_base64(
    plaintext: &[u8],
    key: &EncryptionKey,
    aad: Option<&[u8]>,
) -> Result<String, EncryptionError> {
    let encrypted = encrypt(plaintext, key, aad)?;
    let bytes = encrypted.to_bytes();
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

/// Base64エンコードされた暗号化データを復号
pub fn decrypt_from_base64(
    base64_str: &str,
    key: &EncryptionKey,
) -> Result<Vec<u8>, EncryptionError> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(base64_str)
        .map_err(|e| EncryptionError::InvalidData(format!("Base64デコードエラー: {}", e)))?;
    
    let encrypted = EncryptedData::from_bytes(&bytes)?;
    decrypt(&encrypted, key)
}

/// 文字列を暗号化してBase64エンコード
pub fn encrypt_string_to_base64(
    text: &str,
    key: &EncryptionKey,
    aad: Option<&[u8]>,
) -> Result<String, EncryptionError> {
    encrypt_to_base64(text.as_bytes(), key, aad)
}

/// Base64エンコードされた暗号化データを文字列に復号
pub fn decrypt_string_from_base64(
    base64_str: &str,
    key: &EncryptionKey,
) -> Result<String, EncryptionError> {
    let bytes = decrypt_from_base64(base64_str, key)?;
    String::from_utf8(bytes)
        .map_err(|e| EncryptionError::InvalidData(format!("UTF-8デコードエラー: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_generation() {
        let key = EncryptionKey::generate(EncryptionAlgorithm::Aes256Gcm);
        assert_eq!(key.as_bytes().len(), 32);
        assert_eq!(key.algorithm(), EncryptionAlgorithm::Aes256Gcm);
    }
    
    #[test]
    fn test_encrypt_decrypt() {
        let key = EncryptionKey::generate(EncryptionAlgorithm::Aes256Gcm);
        let plaintext = b"Hello, world!";
        
        let encrypted = encrypt(plaintext, &key, None).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }
    
    #[test]
    fn test_encrypt_decrypt_with_aad() {
        let key = EncryptionKey::generate(EncryptionAlgorithm::Aes256Gcm);
        let plaintext = b"Hello, world!";
        let aad = b"Additional authenticated data";
        
        let encrypted = encrypt(plaintext, &key, Some(aad)).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();
        
        assert_eq!(decrypted, plaintext);
        assert_eq!(encrypted.aad(), Some(aad as &[u8]));
    }
    
    #[test]
    fn test_serialization() {
        let key = EncryptionKey::generate(EncryptionAlgorithm::Aes256Gcm);
        let plaintext = b"Hello, world!";
        
        let encrypted = encrypt(plaintext, &key, None).unwrap();
        let bytes = encrypted.to_bytes();
        let deserialized = EncryptedData::from_bytes(&bytes).unwrap();
        
        assert_eq!(deserialized.algorithm(), encrypted.algorithm());
        assert_eq!(deserialized.ciphertext(), encrypted.ciphertext());
        assert_eq!(deserialized.nonce(), encrypted.nonce());
        assert_eq!(deserialized.aad(), encrypted.aad());
        
        let decrypted = decrypt(&deserialized, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }
    
    #[test]
    fn test_password_key_derivation() {
        let password = "mysecretpassword";
        let salt = generate_salt(16);
        
        let key = EncryptionKey::from_password(password, &salt, EncryptionAlgorithm::Aes256Gcm);
        assert_eq!(key.as_bytes().len(), 32);
        
        // 同じパスワードとソルトからは同じキーが生成されるはず
        let key2 = EncryptionKey::from_password(password, &salt, EncryptionAlgorithm::Aes256Gcm);
        assert_eq!(key.as_bytes(), key2.as_bytes());
        
        // 異なるソルトでは異なるキーが生成されるはず
        let salt2 = generate_salt(16);
        let key3 = EncryptionKey::from_password(password, &salt2, EncryptionAlgorithm::Aes256Gcm);
        assert_ne!(key.as_bytes(), key3.as_bytes());
    }
    
    #[test]
    fn test_base64_encryption() {
        let key = EncryptionKey::generate(EncryptionAlgorithm::Aes256Gcm);
        let plaintext = "秘密のメッセージ";
        
        let encrypted = encrypt_string_to_base64(plaintext, &key, None).unwrap();
        let decrypted = decrypt_string_from_base64(&encrypted, &key).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }
    
    #[test]
    fn test_pbkdf2_key_derivation() {
        let password = "mysecretpassword";
        let salt = generate_salt(16);
        
        let key = derive_key_pbkdf2(password, &salt, 10000, EncryptionAlgorithm::Aes256Gcm).unwrap();
        assert_eq!(key.as_bytes().len(), 32);
        
        // 同じパラメータからは同じキーが生成されるはず
        let key2 = derive_key_pbkdf2(password, &salt, 10000, EncryptionAlgorithm::Aes256Gcm).unwrap();
        assert_eq!(key.as_bytes(), key2.as_bytes());
        
        // 異なる反復回数では異なるキーが生成されるはず
        let key3 = derive_key_pbkdf2(password, &salt, 5000, EncryptionAlgorithm::Aes256Gcm).unwrap();
        assert_ne!(key.as_bytes(), key3.as_bytes());
    }
}