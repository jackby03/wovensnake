use once_cell::sync::Lazy;

/// Shared HTTP client for all outbound requests (`PyPI`, GitHub, Python downloads).
///
/// Reusing a single [`reqwest::Client`] enables TCP connection pooling and
/// HTTP keep-alive, which significantly reduces overhead when installing
/// packages with many dependencies.
pub static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .user_agent("wovensnake")
        .build()
        .expect("failed to build reqwest::Client")
});
