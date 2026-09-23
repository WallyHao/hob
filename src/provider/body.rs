// --- provider::body ---
// Reading a response body, bounded, with the API's own message on a failure.

use super::error::Error;
use super::secret::Secret;

/// The most of one answer that is held in memory.
pub(super) const LIMIT: usize = 10 * 1024 * 1024;

/// Read the body in chunks so a runaway gateway cannot grow the heap past the
/// cap; an error body is only ever shown for its first characters, so it is
/// truncated rather than refused.
pub(super) async fn read(
    provider: &str,
    key: &Secret,
    mut response: reqwest::Response,
) -> Result<String, Error> {
    let status = response.status();
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|source| Error::Http {
        provider: provider.to_owned(),
        source,
    })? {
        if bytes.len() + chunk.len() > LIMIT {
            if status.is_success() {
                return Err(Error::BodyTooLarge {
                    provider: provider.to_owned(),
                    limit: LIMIT,
                });
            }
            bytes.extend_from_slice(&chunk[..LIMIT - bytes.len()]);
            break;
        }
        bytes.extend_from_slice(&chunk);
    }
    let body = String::from_utf8_lossy(&bytes).into_owned();
    if !status.is_success() {
        return Err(Error::api(provider, status.as_u16(), &body, key));
    }
    Ok(body)
}
