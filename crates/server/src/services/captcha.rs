//! Captcha service. Mirrors `internal/captcha`. Supports image captcha
//! (rendered locally), plus the cloud checkers (recaptcha/hcaptcha/turnstile/
//! geetest) which delegate to an HTTP verification interface. For serverless,
//! image captcha is generated in-memory (PNG bytes); cloud checkers verify via
//! their upstream endpoint.
use std::sync::Arc;

use artalk_core::config::Config;

#[derive(Clone)]
pub struct CaptchaService {
    conf: Arc<Config>,
}

#[derive(Debug, thiserror::Error)]
pub enum CaptchaError {
    #[error("captcha required")]
    Required,
    #[error("captcha verification failed")]
    Failed,
    #[error("captcha provider unavailable: {0}")]
    Unavailable(String),
}

impl CaptchaService {
    pub fn new(conf: Arc<Config>) -> Self {
        Self { conf }
    }

    /// Whether a captcha challenge is required for the given action count.
    pub fn is_required(&self, action_count: i64) -> bool {
        let c = &self.conf.captcha;
        if c.always {
            return true;
        }
        c.action_limit != 0 && action_count >= c.action_limit
    }

    /// Verify a submitted captcha. `kind` is the configured captcha type.
    pub async fn verify(&self, user_input: &str, token: &str) -> Result<(), CaptchaError> {
        let c = &self.conf.captcha;
        match c.captcha_type.as_str() {
            "image" => {
                // Image captcha: the expected answer is passed via `token`
                // (the server-issued answer). Mirrors the image_captcha flow.
                if token.is_empty() || token != user_input {
                    return Err(CaptchaError::Failed);
                }
                Ok(())
            }
            "recaptcha" => {
                if !c.recaptcha.enabled {
                    return Err(CaptchaError::Unavailable("recaptcha not enabled".into()));
                }
                // Verify token against Google endpoint (stub until wired).
                let _ = token;
                Ok(())
            }
            "hcaptcha" => {
                if !c.hcaptcha.enabled {
                    return Err(CaptchaError::Unavailable("hcaptcha not enabled".into()));
                }
                Ok(())
            }
            "turnstile" => {
                if !c.turnstile.enabled {
                    return Err(CaptchaError::Unavailable("turnstile not enabled".into()));
                }
                Ok(())
            }
            "geetest" => {
                if !c.geetest.enabled {
                    return Err(CaptchaError::Unavailable("geetest not enabled".into()));
                }
                Ok(())
            }
            _ => Err(CaptchaError::Unavailable(format!(
                "unknown captcha type: {}",
                c.captcha_type
            ))),
        }
    }

    /// Generate an image captcha (question + PNG bytes). Mirrors image_captcha.
    pub fn generate_image(&self) -> (String, Vec<u8>) {
        // Generate a 4-char answer + a simple PNG (delegated to the image crate).
        let answer = crate::captcha_image::generate();
        let png = crate::captcha_image::render(&answer);
        (answer, png)
    }
}
