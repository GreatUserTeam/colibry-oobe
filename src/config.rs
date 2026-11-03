use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::ErrorKind;

/// Configuration collected by the OOBE wizard.
///
/// Passwords are never stored in plain text:
/// - `user_password_hash` and `device_password_hash` are Argon2id PHC strings
///   that the installer can write straight into /etc/shadow.
/// - `wifi_password_encrypted` is AES-256-GCM; the key is derived from
///   SHA-256(device_password_hash || wifi_key_salt), so the installer needs
///   the original device password to decrypt it.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    /// Format version the installer must check for compatibility.
    pub config_version: String,

    /// Locale code, for example "ru" or "de".
    pub language: String,
    /// Console/X11 layouts, derived from the language plus "us" fallback.
    pub keyboard_layout: String,
    /// IANA timezone name, for example "Europe/Moscow".
    pub timezone: String,

    pub user_name: String,
    pub user_login: String,
    pub user_password_hash: String,

    pub hostname: String,
    pub device_password_hash: String,

    pub wifi_ssid: String,
    /// base64(nonce || ciphertext+tag); empty when Wi-Fi was skipped.
    pub wifi_password_encrypted: String,
    pub wifi_key_salt: String,

    pub http_proxy: String,
    pub https_proxy: String,
    pub no_proxy: String,

    /// Unix timestamp of when the wizard finished.
    pub configured_at: i64,
}

impl Config {
    /// Write the config as JSON, preferring /etc and falling back to the
    /// current directory when there are no write permissions.
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let mut last_err = std::io::Error::from(ErrorKind::Other);
        for path in ["/etc/oobe.config.json", "oobe.config.json"] {
            match self.write_file(path) {
                Ok(()) => return Ok(()),
                Err(e) => last_err = e,
            }
        }
        Err(last_err.into())
    }

    /// Serialize to pretty JSON and write one file with owner-only permissions.
    fn write_file(&self, path: &str) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        fs::write(path, json)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }
}
