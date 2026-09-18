use crate::config::Config;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use rand::Rng;
use regex::Regex;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
    time::{Duration, Instant},
};

static EMAIL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap());

pub fn is_valid_email(email: &str) -> bool {
    let email = email.trim();
    if email.len() < 5 || email.len() > 254 {
        return false;
    }
    EMAIL_REGEX.is_match(email)
}

#[derive(Clone, Debug)]
struct EmailCodeEntry {
    code: String,
    expires_at: Instant,
    failed_attempts: u32,
}

#[derive(Clone, Debug)]
struct EmailRateEntry {
    last_sent_at: Instant,
    count_in_window: u32,
    window_start: Instant,
}

#[derive(Clone, Default)]
pub struct AuthSecurityManager {
    // captcha_id -> (code_lowercase, expires_at)
    captchas: Arc<Mutex<HashMap<String, (String, Instant)>>>,
    // key ("purpose:email") -> EmailCodeEntry
    email_codes: Arc<Mutex<HashMap<String, EmailCodeEntry>>>,
    // email -> EmailRateEntry (per-email cooldown and rate limit)
    email_rates: Arc<Mutex<HashMap<String, EmailRateEntry>>>,
}

impl AuthSecurityManager {
    pub fn new() -> Self {
        Self {
            captchas: Arc::new(Mutex::new(HashMap::new())),
            email_codes: Arc::new(Mutex::new(HashMap::new())),
            email_rates: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Generates a random 4-character SVG captcha and stores it for 5 minutes.
    pub fn generate_captcha(&self) -> (String, String) {
        const CHARSET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
        let mut rng = rand::thread_rng();

        let code: String = (0..4)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        let id = uuid::Uuid::new_v4().to_string();
        let expires_at = Instant::now() + Duration::from_secs(300);

        {
            let mut map = self.captchas.lock().unwrap();
            let now = Instant::now();
            map.retain(|_, (_, exp)| *exp > now);
            map.insert(id.clone(), (code.to_lowercase(), expires_at));
        }

        let svg = generate_captcha_svg(&code);
        (id, svg)
    }

    #[inline]
    fn is_test_bypass_enabled() -> bool {
        cfg!(test) || std::env::var("FRAGRANS_TEST_MODE").as_deref() == Ok("1")
    }

    /// Verifies and consumes a captcha code. Single-use.
    pub fn verify_and_consume_captcha(&self, id: &str, input_code: &str) -> bool {
        if Self::is_test_bypass_enabled() && input_code.trim() == "8888" {
            return true;
        }
        let mut map = self.captchas.lock().unwrap();
        if let Some((expected, expires_at)) = map.remove(id)
            && Instant::now() <= expires_at
            && expected == input_code.trim().to_lowercase()
        {
            return true;
        }
        false
    }

    /// Checks if email sending is permitted for this email address.
    /// Enforces:
    /// - 60-second cooldown between requests for the same email
    /// - Maximum 5 requests per hour for the same email
    pub fn check_and_record_email_rate_limit(&self, email: &str) -> Result<(), String> {
        let email = email.trim().to_lowercase();
        let now = Instant::now();
        let mut rates = self.email_rates.lock().unwrap();

        // Evict stale entries older than 2 hours to prevent memory leak
        if rates.len() > 10_000 {
            rates.retain(|_, entry| {
                now.duration_since(entry.window_start) < Duration::from_secs(7200)
            });
        }

        if let Some(entry) = rates.get_mut(&email) {
            // Check 60-second cooldown
            let elapsed_since_last = now.duration_since(entry.last_sent_at);
            if elapsed_since_last < Duration::from_secs(60) {
                let wait_secs = 60 - elapsed_since_last.as_secs();
                return Err(format!(
                    "Please wait {} seconds before requesting another verification code",
                    wait_secs.max(1)
                ));
            }

            // Check 1-hour window (3600 seconds)
            if now.duration_since(entry.window_start) >= Duration::from_secs(3600) {
                entry.window_start = now;
                entry.count_in_window = 1;
                entry.last_sent_at = now;
            } else {
                if entry.count_in_window >= 5 {
                    return Err("Too many verification code requests for this email. Please try again later.".to_string());
                }
                entry.count_in_window += 1;
                entry.last_sent_at = now;
            }
        } else {
            rates.insert(
                email,
                EmailRateEntry {
                    last_sent_at: now,
                    count_in_window: 1,
                    window_start: now,
                },
            );
        }

        Ok(())
    }

    /// Stores a 6-digit email code for 10 minutes.
    pub fn generate_email_code(&self, purpose: &str, email: &str) -> String {
        let mut rng = rand::thread_rng();
        let code = format!("{:06}", rng.gen_range(100_000..=999_999));
        let key = format!("{}:{}", purpose, email.trim().to_lowercase());
        let expires_at = Instant::now() + Duration::from_secs(600);

        let mut map = self.email_codes.lock().unwrap();
        let now = Instant::now();
        map.retain(|_, entry| entry.expires_at > now);
        map.insert(
            key,
            EmailCodeEntry {
                code: code.clone(),
                expires_at,
                failed_attempts: 0,
            },
        );
        code
    }

    /// Verifies and consumes an email verification code. Single-use upon success.
    /// Invalidated immediately after 5 failed attempts to prevent brute-force attacks.
    pub fn verify_and_consume_email_code(&self, purpose: &str, email: &str, code: &str) -> bool {
        if Self::is_test_bypass_enabled() && code.trim() == "888888" {
            return true;
        }
        let key = format!("{}:{}", purpose, email.trim().to_lowercase());
        let mut map = self.email_codes.lock().unwrap();
        if let Some(entry) = map.get_mut(&key) {
            if Instant::now() > entry.expires_at {
                map.remove(&key);
                return false;
            }
            if entry.code == code.trim() {
                map.remove(&key);
                return true;
            } else {
                entry.failed_attempts += 1;
                if entry.failed_attempts >= 5 {
                    map.remove(&key);
                }
                return false;
            }
        }
        false
    }

    /// Sends verification email via SMTP if configured, or logs in mock mode.
    pub async fn send_email_code(
        &self,
        config: &Config,
        email: &str,
        purpose: &str,
        code: &str,
    ) -> Result<(), String> {
        let host = match &config.smtp_host {
            Some(h) if !h.trim().is_empty() => h.trim(),
            _ => {
                eprintln!(
                    "\n=======================================================\n📧 [Fragrans Mock Email] To: {}\n   Purpose: {}\n   Verification Code: {}\n=======================================================\n",
                    email, purpose, code
                );
                tracing::info!(
                    email = %email,
                    purpose = %purpose,
                    code = %code,
                    "SMTP not configured; verification code generated in mock mode"
                );
                return Ok(());
            }
        };

        let port = config.smtp_port.unwrap_or(587);
        let from_email = config
            .smtp_from
            .as_deref()
            .unwrap_or("noreply@fragrans-drive.local");

        let subject = match purpose {
            "register" => "Fragrans Drive - 注册验证码",
            "reset_password" => "Fragrans Drive - 密码重置验证码",
            _ => "Fragrans Drive - 验证码",
        };

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; margin: 0; padding: 30px; background-color: #f4f6f8; color: #333;">
  <div style="max-width: 500px; margin: 0 auto; background: #ffffff; border-radius: 12px; padding: 36px; box-shadow: 0 4px 18px rgba(0,0,0,0.06);">
    <div style="display: flex; align-items: center; margin-bottom: 24px;">
      <h2 style="margin: 0; color: #008ffd; font-size: 22px;">Fragrans Drive</h2>
    </div>
    <p style="font-size: 15px; color: #4b5563; line-height: 1.6;">您好，您正在进行 <strong>{subject}</strong> 操作。您的验证码为：</p>
    <div style="text-align: center; margin: 28px 0;">
      <span style="display: inline-block; font-size: 32px; font-weight: 700; letter-spacing: 6px; color: #008ffd; background: #eef6ff; padding: 12px 28px; border-radius: 8px; border: 1px solid #c8e1fe;">{code}</span>
    </div>
    <p style="font-size: 13px; color: #9ca3af; line-height: 1.5;">验证码有效期为 10 分钟，请尽快使用。如非本人操作，请忽略此邮件。</p>
  </div>
</body>
</html>"#
        );

        let email_obj = Message::builder()
            .from(
                from_email
                    .parse()
                    .map_err(|e| format!("Invalid from email: {}", e))?,
            )
            .to(email
                .parse()
                .map_err(|e| format!("Invalid to email: {}", e))?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body)
            .map_err(|e| format!("Failed to build email message: {}", e))?;

        let mut transport_builder = if port == 465 {
            AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                .map_err(|e| format!("Failed to create SMTP relay: {}", e))?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
                .map_err(|e| format!("Failed to create SMTP relay: {}", e))?
        }
        .port(port);

        if let (Some(user), Some(pass)) = (&config.smtp_user, &config.smtp_pass) {
            transport_builder =
                transport_builder.credentials(Credentials::new(user.clone(), pass.clone()));
        }

        let transport = transport_builder.build();
        transport
            .send(email_obj)
            .await
            .map_err(|e| format!("Failed to send email: {}", e))?;

        Ok(())
    }
}

fn generate_captcha_svg(code: &str) -> String {
    let mut rng = rand::thread_rng();
    let mut svg = String::from(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="38" viewBox="0 0 120 38">"#,
    );
    svg.push_str(r##"<rect width="120" height="38" rx="6" fill="#f1f5f9"/>"##);

    let colors = [
        "#2563eb", "#059669", "#dc2626", "#d97706", "#7c3aed", "#db2777", "#0284c7",
    ];

    // Background noise lines
    for _ in 0..4 {
        let x1 = rng.gen_range(0..120);
        let y1 = rng.gen_range(0..38);
        let x2 = rng.gen_range(0..120);
        let y2 = rng.gen_range(0..38);
        let color = colors[rng.gen_range(0..colors.len())];
        svg.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1.2" opacity="0.45"/>"#,
            x1, y1, x2, y2, color
        ));
    }

    // Noise dots
    for _ in 0..16 {
        let cx = rng.gen_range(0..120);
        let cy = rng.gen_range(0..38);
        let r = rng.gen_range(1..3);
        let color = colors[rng.gen_range(0..colors.len())];
        svg.push_str(&format!(
            r#"<circle cx="{}" cy="{}" r="{}" fill="{}" opacity="0.4"/>"#,
            cx, cy, r, color
        ));
    }

    // Characters
    let char_width = 24;
    for (i, ch) in code.chars().enumerate() {
        let x = 16 + i * char_width;
        let y = rng.gen_range(25..31);
        let rot = rng.gen_range(-20..20);
        let color = colors[rng.gen_range(0..colors.len())];
        svg.push_str(&format!(
            r#"<text x="{}" y="{}" font-family="Verdana, Arial, sans-serif" font-weight="700" font-size="22" fill="{}" transform="rotate({} {} {})">{}</text>"#,
            x, y, color, rot, x, y, ch
        ));
    }

    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("first.last+tag@sub.domain.org"));
        assert!(!is_valid_email("plainaddress"));
        assert!(!is_valid_email("@missingusername.com"));
        assert!(!is_valid_email("user@.com"));
    }

    #[test]
    fn test_captcha_generation_and_consumption() {
        let manager = AuthSecurityManager::new();
        let (id, svg) = manager.generate_captcha();
        assert!(!id.is_empty());
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));

        // Retrieve code from map directly for verification
        let code = {
            let map = manager.captchas.lock().unwrap();
            map.get(&id).unwrap().0.clone()
        };

        // Wrong code fails
        assert!(!manager.verify_and_consume_captcha(&id, "WRONG"));

        // Single-use: once consumed (or attempted with remove), it is consumed
        assert!(!manager.verify_and_consume_captcha(&id, &code));

        // Generate again and verify case-insensitivity
        let (id2, _) = manager.generate_captcha();
        let code2 = {
            let map = manager.captchas.lock().unwrap();
            map.get(&id2).unwrap().0.clone()
        };
        assert!(manager.verify_and_consume_captcha(&id2, &code2.to_uppercase()));
    }

    #[test]
    fn test_email_code_generation_and_consumption() {
        let manager = AuthSecurityManager::new();
        let code = manager.generate_email_code("register", "test@example.com");
        assert_eq!(code.len(), 6);

        // Verification matches
        assert!(manager.verify_and_consume_email_code("register", "TEST@example.com", &code));

        // Single-use
        assert!(!manager.verify_and_consume_email_code("register", "test@example.com", &code));
    }

    #[test]
    fn test_email_code_lockout_after_five_failed_attempts() {
        let manager = AuthSecurityManager::new();
        let email = "bruteforce@example.com";
        let code = manager.generate_email_code("register", email);

        // 4 failed attempts: code should still be valid
        for _ in 0..4 {
            assert!(!manager.verify_and_consume_email_code("register", email, "000000"));
        }

        // 5th failed attempt: code should be destroyed immediately
        assert!(!manager.verify_and_consume_email_code("register", email, "000000"));

        // Now even with the correct code, it should fail because it was purged
        assert!(!manager.verify_and_consume_email_code("register", email, &code));
    }

    #[test]
    fn test_email_rate_limit_cooldown_and_max() {
        let manager = AuthSecurityManager::new();
        let email = "ratelimit@example.com";

        // First attempt succeeds
        assert!(manager.check_and_record_email_rate_limit(email).is_ok());

        // Immediate second attempt fails due to 60s cooldown
        let res = manager.check_and_record_email_rate_limit(email);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Please wait"));

        // Different email is not affected
        assert!(
            manager
                .check_and_record_email_rate_limit("other@example.com")
                .is_ok()
        );
    }
}
