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
#[derive(Debug, thiserror::Error)]
pub enum LicenseError {
    #[error("license string format is invalid")]
    Format,
    #[error("license signature mismatch")]
    Signature,
    #[error("license secret not configured (env LICENSE_SECRET or compile-time variable)")]
    MissingSecret,
}

static RUNTIME_SECRET: OnceCell<String> = OnceCell::new();

/// Allow libraries / binaries that link to bstseal-core to set the shared
/// secret at runtime (e.g. via FFI).
/// Returns `true` if the secret was set, `false` if it was already set before.
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

/// Verify a license string and return the encoded tier on success.
///
/// License format: `<uuid>.<tier>.<signature>` where
/// `signature = base64url(HMAC_SHA256("<uuid>.<tier>", LICENSE_SECRET))`
pub fn verify_license(license: &str) -> Result<Tier, LicenseError> {
    let mut parts = license.split('.').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(LicenseError::Format);
    }
    let sig_provided = parts.pop().unwrap();
    let tier_str = parts.pop().unwrap();
    let uuid_part = parts.pop().unwrap();

    let data = format!("{uuid_part}.{tier_str}");
    let secret = get_secret()?;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| LicenseError::MissingSecret)?;
    mac.update(data.as_bytes());
    let expected_sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

    if expected_sig != sig_provided {
        return Err(LicenseError::Signature);
    }

    Ok(Tier::from_str(tier_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hmac::Mac;

    /// helper to generate license inside tests
    fn make_license(tier: &str, secret: &str) -> String {
        let uuid = "123e4567-e89b-12d3-a456-426614174000";
        let data = format!("{uuid}.{tier}");
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(data.as_bytes());
        let sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
        format!("{data}.{sig}")
    }

    #[test]
    fn verify_roundtrip() {
        let secret = "test_secret";
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
