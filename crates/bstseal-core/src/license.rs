use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use once_cell::sync::OnceCell;
use sha2::Sha256;

/// HMAC-SHA256 alias.
type HmacSha256 = Hmac<Sha256>;

/// Available pricing tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Solo,
    Startup,
    Unknown,
}

impl Tier {
    fn from_str(s: &str) -> Self {
        match s {
            "solo" => Tier::Solo,
            "startup" => Tier::Startup,
            _ => Tier::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::Solo => "solo",
            Tier::Startup => "startup",
            Tier::Unknown => "unknown",
        }
    }
}

/// Errors that can occur during license verification.
#[derive(Debug, thiserror::Error, Clone)]
pub enum LicenseError {
    #[error("license string format is invalid")]
    Format,
    #[error("license signature mismatch")]
    Signature,
    #[error("license secret not configured (env LICENSE_SECRET or compile-time variable)")]
    MissingSecret,
    #[error("license key not provided (env BSTSEAL_LICENSE or runtime call)")]
    MissingKey,
    #[error("license key expired")] Expired,
}

static RUNTIME_SECRET: OnceCell<String> = OnceCell::new();
static RUNTIME_LICENSE: OnceCell<String> = OnceCell::new();

/// Allow libraries / binaries that link to bstseal-core to set the shared
/// secret at runtime (e.g. via FFI).
/// Returns `true` if the secret was set, `false` if it was already set before.
/// Set shared HMAC secret at runtime.
pub fn set_license_secret<S: Into<String>>(secret: S) -> bool {
    RUNTIME_SECRET.set(secret.into()).is_ok()
}

/// Obtain license secret from (in order):
/// 1. Runtime call [`set_license_secret`]  
/// 2. Environment variable `LICENSE_SECRET`  
/// 3. Compile-time variable `LICENSE_SECRET` (provided via `cargo rustc --cfg`)
fn get_secret() -> Result<String, LicenseError> {
    if let Some(s) = RUNTIME_SECRET.get() {
        return Ok(s.clone());
    }
    if let Ok(env) = std::env::var("LICENSE_SECRET") {
        return Ok(env);
    }
    if let Some(ct) = option_env!("LICENSE_SECRET") {
        return Ok(ct.to_owned());
    }
    Err(LicenseError::MissingSecret)
}

/// Get license string from runtime, env, or error.
fn get_license() -> Result<String, LicenseError> {
    use std::fs;
    if let Some(k) = RUNTIME_LICENSE.get() {
        return Ok(k.clone());
    }
    if let Ok(env) = std::env::var("BSTSEAL_LICENSE") {
        return Ok(env);
    }
    // fallback to ~/.bstseal/license
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".bstseal").join("license");
        if let Ok(data) = fs::read_to_string(path) {
            let trimmed = data.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }
    Err(LicenseError::MissingKey)
}


/// Verify a license string and return the encoded tier on success.
///
/// License format: `<uuid>.<tier>.<signature>` where
/// `signature = base64url(HMAC_SHA256("<uuid>.<tier>", LICENSE_SECRET))`
pub fn verify_license(license: &str) -> Result<Tier, LicenseError> {
    let parts: Vec<&str> = license.split('.').collect();
    if parts.len() < 4 {
        return Err(LicenseError::Format);
    }
    let sig_provided = *parts.last().unwrap();
    let uuid_part = parts[0];
    let tier_str = parts[1];
    let expires_iso = parts[2..parts.len()-1].join(".");

    let data = format!("{uuid_part}.{tier_str}.{expires_iso}");
    let secret = get_secret()?;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| LicenseError::MissingSecret)?;
    mac.update(data.as_bytes());
    let expected_sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

    if expected_sig != sig_provided {
        return Err(LicenseError::Signature);
    }

    // expiry check
    let expires_at = chrono::DateTime::parse_from_rfc3339(&expires_iso)
        .map_err(|_| LicenseError::Format)?
        .with_timezone(&chrono::Utc);
    if chrono::Utc::now() > expires_at {
        return Err(LicenseError::Expired);
    }

    Ok(Tier::from_str(tier_str))
}

use once_cell::sync::Lazy;
static LICENSE_CHECK: Lazy<Result<Tier, LicenseError>> = Lazy::new(|| {
    let lic = get_license()?;
    verify_license(&lic)
});

/// Ensure license was verified successfully; returns Tier or error.
pub fn ensure_license_valid() -> Result<Tier, LicenseError> {
    (*LICENSE_CHECK).clone()
}

/// Set license key at runtime.
pub fn set_license_key<S: Into<String>>(key: S) -> bool {
    RUNTIME_LICENSE.set(key.into()).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hmac::Mac;

    /// helper to generate license inside tests
    fn make_license(tier: &str, secret: &str) -> String {
        use chrono::{Duration, Utc};
        let uuid = "123e4567-e89b-12d3-a456-426614174000";
        let expires = (Utc::now() + Duration::days(365)).to_rfc3339();
        let data = format!("{uuid}.{tier}.{expires}");
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(data.as_bytes());
        let sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
        format!("{data}.{sig}")
    }

    #[test]
    fn verify_roundtrip() {
        let secret = "abc";
        set_license_secret(secret.to_string());
        let lic = make_license("solo", secret);
        let tier = verify_license(&lic).unwrap();
        assert_eq!(tier, Tier::Solo);
    }

    #[test]
    fn fail_on_wrong_sig() {
        set_license_secret("abc".to_string());
        let lic = "bad.license.signature";
        assert!(verify_license(lic).is_err());
    }
}
