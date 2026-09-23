// --- provider::stream ---
// Reading an SSE body. Framing lives here, the meaning of each event lives in
// the dialect decoder, and text reaches `sink` as it arrives.

use reqwest::Response;

use super::error::Error;
use super::spec::ProviderSpec;
use super::types::ChatResponse;
use super::wire;

/// The most one streamed answer may bring in, matching the non-streaming cap.
const LIMIT: usize = 10 * 1024 * 1024;

/// Read an SSE body to its end, showing each text delta.
pub(crate) async fn read(
    mut response: Response,
    spec: &ProviderSpec,
    sink: &mut dyn FnMut(&str),
) -> Result<ChatResponse, Error> {
    let mut decoder = wire::decoder(spec);
    let mut buffer = String::new();
    let mut total = 0usize;
    let mut done = false;
    while !done {
        let chunk = response.chunk().await.map_err(|source| Error::Http {
            provider: spec.id.clone(),
            source,
        })?;
        let Some(chunk) = chunk else { break };
        total += chunk.len();
        if total > LIMIT {
            return Err(Error::BodyTooLarge {
                provider: spec.id.clone(),
                limit: LIMIT,
            });
        }
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(end) = buffer.find('\n') {
            let line: String = buffer.drain(..=end).collect();
            let Some(data) = line.trim_end_matches(['\r', '\n']).strip_prefix("data:") else {
                continue;
            };
            match data.trim() {
                "[DONE]" => {
                    done = true;
                    break;
                }
                data => {
                    if let Some(delta) = decoder.feed(data)? {
                        sink(&delta);
                    }
                }
            }
        }
    }
    decoder.finish(&spec.id)
}
