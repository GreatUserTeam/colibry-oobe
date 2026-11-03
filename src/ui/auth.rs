//! ColibryID account verification against the colibryid.guteam.net server.

/// Result of a verification attempt.
pub enum Outcome {
    /// HTTP 2xx: the server accepted the credentials.
    Verified,
    /// HTTP 401/403: the server rejected this email/password pair.
    WrongCredentials,
    /// Network failure, timeout or an unexpected HTTP status.
    Unavailable,
}

/// Ask the ColibryID endpoint to verify the credentials. The URL and the
/// form field names are the single place to adjust when the ColibryID API
/// contract is finalized.
pub fn verify(email: &str, password: &str) -> Outcome {
    let mut cmd = std::process::Command::new("curl");
    cmd.arg("-sS")
        .arg("-o")
        .arg("/dev/null")
        .arg("-w")
        .arg("%{http_code}")
        .arg("--max-time")
        .arg("12")
        .arg("-X")
        .arg("POST")
        .arg("--data-urlencode")
        .arg(format!("email={email}"))
        .arg("--data-urlencode")
        .arg(format!("password={password}"))
        .arg("https://colibryid.guteam.net/");
    let out = match cmd.output() {
        Ok(out) if out.status.success() => out,
        _ => return Outcome::Unavailable,
    };
    let code = String::from_utf8_lossy(&out.stdout);
    if code.starts_with('2') {
        Outcome::Verified
    } else if code.starts_with("401") || code.starts_with("403") {
        Outcome::WrongCredentials
    } else {
        Outcome::Unavailable
    }
}
