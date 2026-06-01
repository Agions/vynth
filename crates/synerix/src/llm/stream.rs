//! SSE stream parser for OpenAI-compatible APIs
//!
//! Performance optimizations:
//! - `BytesMut` buffer: zero-copy append, efficient drain
//! - Avoids repeated String allocations on each chunk
//! - Pre-allocated JSON parsing with `serde_json::from_slice`

use bytes::{Buf, BytesMut};
use futures::stream::Stream;
use futures::stream::StreamExt;

use crate::error::AppError;
use crate::llm::types::{ChunkDelta, StreamChunk};

/// Initial buffer capacity for SSE parsing (avoids early re-allocation)
const INITIAL_BUFFER_CAPACITY: usize = 4096;

/// Parse an SSE byte stream into StreamChunks
///
/// Optimized: uses `BytesMut` buffer to minimize allocations.
/// Each SSE event is parsed directly from the buffer slice without
/// intermediate String creation.
pub fn parse_sse_stream<S>(byte_stream: S) -> impl Stream<Item = Result<StreamChunk, AppError>>
where
    S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + Unpin,
{
    async_stream::stream! {
        let mut buffer = BytesMut::with_capacity(INITIAL_BUFFER_CAPACITY);
        let mut stream = byte_stream;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| AppError::Llm(e.to_string()))?;
            buffer.extend_from_slice(&chunk);

            // Process complete SSE events (delimited by double newline)
            while let Some(event_end) = find_double_newline(&buffer) {
                // Extract event data as a slice (no allocation)
                let event_slice = &buffer[..event_end];

                // Collect lines first to release immutable borrow before advancing
                let lines: Vec<&[u8]> = split_lines(event_slice).collect();

                let mut should_return = false;
                for line in &lines {
                    // Check for "data: " prefix (6 bytes)
                    if line.len() > 6 && &line[..6] == b"data: " {
                        let data = trim_ascii(&line[6..]);

                        // Check for [DONE]
                        if data == b"[DONE]" {
                            yield Ok(StreamChunk { delta: ChunkDelta::Done });
                            should_return = true;
                            break;
                        }

                        // Parse JSON directly from bytes (avoids String allocation)
                        if let Some(chunk) = parse_data_bytes(data) {
                            yield Ok(chunk);
                        }
                    }
                }

                // Drain processed event + delimiter (safe: lines Vec is consumed)
                buffer.advance(event_end + 2);

                if should_return {
                    return;
                }
            }

            // Prevent unbounded buffer growth (truncate if > 256KB without delimiter)
            if buffer.len() > 256 * 1024 {
                tracing::warn!("SSE buffer overflow ({} bytes), force-processing", buffer.len());
                // Try to parse whatever we have
                if let Some(chunk) = parse_remaining_buffer(&buffer) {
                    yield Ok(chunk);
                }
                buffer.clear();
            }
        }

        // Process any remaining data in buffer
        if !buffer.is_empty() {
            for line in split_lines(&buffer) {
                if line.len() > 6 && &line[..6] == b"data: " {
                    let data = trim_ascii(&line[6..]);
                    if data == b"[DONE]" {
                        yield Ok(StreamChunk { delta: ChunkDelta::Done });
                        return;
                    }
                    if let Some(chunk) = parse_data_bytes(data) {
                        yield Ok(chunk);
                    }
                }
            }
        }

        yield Ok(StreamChunk { delta: ChunkDelta::Done });
    }
}

/// Split bytes by newline — returns iterator of `&[u8]` slices
///
/// Works around `BytesMut::split` not supporting predicate-based splitting.
#[inline]
fn split_lines(data: &[u8]) -> impl Iterator<Item = &[u8]> {
    // Use standard slice split
    data.split(|&b| b == b'\n')
}

/// Find double newline (`\n\n`) in buffer — returns the index of the first `\n`
#[inline]
fn find_double_newline(buffer: &[u8]) -> Option<usize> {
    let len = buffer.len();
    if len < 2 {
        return None;
    }

    // Scan for \n\n pattern
    (0..len - 1).find(|&i| buffer[i] == b'\n' && buffer[i + 1] == b'\n')
}

/// Trim ASCII whitespace from both ends
#[inline]
fn trim_ascii(data: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = data.len();

    while start < end && data[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > start && data[end - 1].is_ascii_whitespace() {
        end -= 1;
    }

    &data[start..end]
}

/// Parse a single SSE data line directly from bytes (avoids String allocation)
fn parse_data_bytes(data: &[u8]) -> Option<StreamChunk> {
    // Use from_slice to parse JSON directly from bytes
    let parsed: serde_json::Value = serde_json::from_slice(data).ok()?;
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
        if let Some(tc) = tool_calls.iter().next() {
            let id = tc.get("id")?.as_str()?.to_string();
            let name = tc.get("function")?.get("name")?.as_str()?.to_string();
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

/// Try to parse remaining buffer content (overflow fallback)
fn parse_remaining_buffer(buffer: &[u8]) -> Option<StreamChunk> {
    // Try the whole buffer as a JSON line
    if let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(buffer) {
        let delta = parsed.get("choices")?.get(0)?.get("delta")?;
        if let Some(content) = delta.get("content").and_then(|v| v.as_str()) {
            if !content.is_empty() {
                return Some(StreamChunk {
                    delta: ChunkDelta::Text {
                        content: content.to_string(),
                    },
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_double_newline() {
        assert_eq!(find_double_newline(b"hello\n\nworld"), Some(5));
        assert_eq!(find_double_newline(b"hello\nworld"), None);
        assert_eq!(find_double_newline(b"\n\n"), Some(0));
        assert_eq!(find_double_newline(b"a"), None);
        assert_eq!(find_double_newline(b""), None);
    }

    #[test]
    fn test_trim_ascii() {
        assert_eq!(trim_ascii(b"  hello  "), b"hello");
        assert_eq!(trim_ascii(b"\t\nhello\r\n"), b"hello");
        assert_eq!(trim_ascii(b"no trim"), b"no trim");
        assert_eq!(trim_ascii(b"   "), b"");
    }

    #[test]
    fn test_split_lines() {
        let data = b"line1\nline2\nline3";
        let lines: Vec<&[u8]> = split_lines(data).collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], b"line1");
        assert_eq!(lines[1], b"line2");
        assert_eq!(lines[2], b"line3");
    }

    #[test]
    fn test_parse_data_bytes_text() {
        let data = br#"{"choices":[{"delta":{"content":"Hello"}}]}"#;
        let chunk = parse_data_bytes(data).unwrap();
        match chunk.delta {
            ChunkDelta::Text { content } => assert_eq!(content, "Hello"),
            _ => panic!("Expected Text delta"),
        }
    }

    #[test]
    fn test_parse_data_bytes_tool_call() {
        let data = br#"{"choices":[{"delta":{"tool_calls":[{"id":"call_1","function":{"name":"read_file","arguments":"{\"path\":\"test.rs\"}"}}]}}]}"#;
        let chunk = parse_data_bytes(data).unwrap();
        match chunk.delta {
            ChunkDelta::ToolCall { id, name, .. } => {
                assert_eq!(id, "call_1");
                assert_eq!(name, "read_file");
            }
            _ => panic!("Expected ToolCall delta"),
        }
    }

    #[test]
    fn test_parse_data_bytes_done() {
        assert!(parse_data_bytes(b"[DONE]").is_none());
    }

    #[test]
    fn test_parse_data_bytes_empty_content() {
        let data = br#"{"choices":[{"delta":{"content":""}}]}"#;
        assert!(parse_data_bytes(data).is_none());
    }
}
