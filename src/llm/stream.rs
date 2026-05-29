//! SSE stream parser for OpenAI-compatible APIs

use futures::stream::Stream;
use futures::stream::StreamExt;

use crate::error::AppError;
use crate::llm::types::{ChunkDelta, StreamChunk};

/// Parse an SSE byte stream into StreamChunks
pub fn parse_sse_stream<S>(byte_stream: S) -> impl Stream<Item = Result<StreamChunk, AppError>>
where
    S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + Unpin,
{
    async_stream::stream! {
        let mut buffer = String::new();
        let mut stream = byte_stream;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| AppError::Llm(e.to_string()))?;
            let text = String::from_utf8_lossy(&chunk);
            buffer.push_str(&text);

            // Process complete SSE events (delimited by double newline)
            while let Some(event_end) = buffer.find("\n\n") {
                let event_data = buffer[..event_end].to_string();
                buffer = buffer[event_end + 2..].to_string();

                for line in event_data.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        let data = data.trim();
                        if data == "[DONE]" {
                            yield Ok(StreamChunk { delta: ChunkDelta::Done });
                            return;
                        }

                        if let Some(chunk) = parse_data_line(data) {
                            yield Ok(chunk);
                        }
                    }
                }
            }
        }

        // Process any remaining data in buffer
        if !buffer.is_empty() {
            for line in buffer.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    let data = data.trim();
                    if data == "[DONE]" {
                        yield Ok(StreamChunk { delta: ChunkDelta::Done });
                        return;
                    }

                    if let Some(chunk) = parse_data_line(data) {
                        yield Ok(chunk);
                    }
                }
            }
        }

        yield Ok(StreamChunk { delta: ChunkDelta::Done });
    }
}

/// Parse a single SSE data line
fn parse_data_line(data: &str) -> Option<StreamChunk> {
    let parsed: serde_json::Value = serde_json::from_str(data).ok()?;
    let delta = parsed.get("choices")?.get(0)?.get("delta")?;

    // Text content
    if let Some(content) = delta.get("content").and_then(|v| v.as_str()) {
        if !content.is_empty() {
            return Some(StreamChunk {
                delta: ChunkDelta::Text {
                    content: content.to_string(),
                },
            });
        }
    }

    // Tool call (incremental)
    if let Some(tool_calls) = delta.get("tool_calls").and_then(|v| v.as_array()) {
        for tc in tool_calls {
            let id = tc.get("id")?.as_str()?.to_string();
            let name = tc
                .get("function")?
                .get("name")?
                .as_str()?
                .to_string();
            let args_delta = tc
                .get("function")?
                .get("arguments")?
                .as_str()
                .unwrap_or("")
                .to_string();

            return Some(StreamChunk {
                delta: ChunkDelta::ToolCall {
                    id,
                    name,
                    args_delta,
                },
            });
        }
    }

    None
}
