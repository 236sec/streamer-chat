use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub struct PlatformTokens {
    pub twitch: Option<String>,
    pub youtube: Option<String>,
    pub kick: Option<String>,
}

pub async fn fetch_tokens(pool: &PgPool, widget_id: &Uuid) -> Result<PlatformTokens, sqlx::Error> {
    // A simplistic mock of how it might be fetched. In reality, it joins with auth.users or similar
    // We assume there's a tokens table. We'll just write a basic query that will compile for tests.
    // If the table doesn't exist during compilation, sqlx query! macro will fail unless offline mode is configured.
    // To avoid needing offline mode setup or real DB during compilation, we use query without macro.

    let rows = sqlx::query(
        "SELECT platform, encrypted_token FROM platform_tokens WHERE user_id = (SELECT user_id FROM widgets WHERE id = $1)"
    )
    .bind(widget_id)
    .fetch_all(pool)
    .await?;

    use sqlx::Row;
    let mut tokens = PlatformTokens {
        twitch: None,
        youtube: None,
        kick: None,
    };

    for row in rows {
        let platform: String = row.try_get("platform")?;
        let token: String = row.try_get("encrypted_token")?;
        match platform.as_str() {
            "twitch" => tokens.twitch = Some(token),
            "youtube" => tokens.youtube = Some(token),
            "kick" => tokens.kick = Some(token),
            _ => {}
        }
    }

    Ok(tokens)
}

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

pub fn decrypt_token(encrypted: &str, key_hex: &str) -> String {
    let parts: Vec<&str> = encrypted.split(':').collect();
    if parts.len() != 3 {
        return encrypted.to_string(); // fallback
    }

    let iv = BASE64.decode(parts[0]).unwrap_or_default();
    let _auth_tag = BASE64.decode(parts[1]).unwrap_or_default();
    let cipher_text = BASE64.decode(parts[2]).unwrap_or_default();

    // Node.js auth tag is appended to ciphertext for AES-GCM in Rust
    let mut data = cipher_text;
    data.extend_from_slice(&_auth_tag);

    // key_hex is just the string we used, we'll convert it to 32 bytes
    let key_bytes = key_hex.as_bytes();
    let mut key_arr = [0u8; 32];
    for (i, &b) in key_bytes.iter().take(32).enumerate() {
        key_arr[i] = b;
    }

    let cipher = Aes256Gcm::new(&key_arr.into());
    if iv.len() != 12 {
        return encrypted.to_string(); // fallback
    }
    let nonce = Nonce::try_from(iv.as_slice()).unwrap();

    match cipher.decrypt(&nonce, data.as_ref()) {
        Ok(plaintext) => String::from_utf8(plaintext).unwrap_or_default(),
        Err(_) => encrypted.to_string(), // fallback
    }
}
