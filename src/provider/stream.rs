// --- provider::stream ---
// Reading an SSE body. Framing lives here, the meaning of each event lives in
// the dialect decoder, and text reaches `sink` as it arrives.

use reqwest::Response;

use super::body::LIMIT;
use super::error::Error;
use super::spec::ProviderSpec;
use super::sse::Events;
use super::types::ChatResponse;
use super::wire;

/// Read an SSE body to its end, showing each text delta.
pub(crate) async fn read(
    mut response: Response,
    spec: &ProviderSpec,
    sink: &mut dyn FnMut(&str),
) -> Result<ChatResponse, Error> {
    let mut decoder = wire::decoder(spec);
    let mut events = Events::default();
    let mut total = 0usize;
    loop {
        let chunk = response.chunk().await.map_err(|source| Error::Http {
            provider: spec.id.clone(),
            source,
        })?;
        let Some(chunk) = chunk else {
            return Err(Error::Shape {
                provider: spec.id.clone(),
                message: "event stream ended before its completion marker".to_owned(),
            });
        };
        total += chunk.len();
        if total > LIMIT {
            return Err(Error::BodyTooLarge {
                provider: spec.id.clone(),
                limit: LIMIT,
            });
        }
        for data in events.push(&chunk).map_err(|message| Error::Shape {
            provider: spec.id.clone(),
            message,
        })? {
            if data.trim() == "[DONE]" {
                if spec.protocol != super::Protocol::OpenAi {
                    return Err(Error::Shape {
                        provider: spec.id.clone(),
                        message: "unexpected completion marker".to_owned(),
                    });
                }
                return decoder.finish(&spec.id);
            }
            if let Some(delta) = decoder.feed(&data)? {
                sink(&delta);
            }
            if decoder.complete() {
                return decoder.finish(&spec.id);
            }
        }
    }
}
