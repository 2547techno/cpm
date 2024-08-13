use pretty_duration::pretty_duration;
use reqwest::{blocking::Response, header::HeaderValue};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn handle_github_rate_limit(response: &Response) -> Result<(), String> {
    let status = response.status();
    if status.as_u16() == 403 || status.as_u16() == 429 {
        // rate limit reached
        let default_header_value = HeaderValue::from_str("").unwrap();
        let reset_epoch = response
            .headers()
            .get("X-RateLimit-Reset")
            .unwrap_or(&default_header_value)
            .to_str()
            .unwrap_or("")
            .parse::<i32>()
            .unwrap_or(-1);
        let current_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32;

        let mut error_str = "GitHub API rate limit reached!".to_owned();
        let duration_str = pretty_duration(
            &Duration::from_secs((reset_epoch - current_epoch) as u64),
            None,
        );
        if reset_epoch != -1 {
            error_str.push_str(&format!(" Resets in {}", duration_str));
        }

        return Err(error_str);
    } else if !status.is_success() {
        let status_str = status.as_str();
        return Err(format!(
            "GitHub API returned an unexpected status code: {status_str}"
        ));
    }

    Ok(())
}
