//! # sabr-rs
//!
//! Rust implementation of YouTube's SABR (Server Adaptive Bit Rate) streaming
//! protocol. Takes a YouTube video ID and streams back audio bytes (WebM/Opus).
//!
//! SABR is the protocol YouTube uses internally for adaptive streaming. It
//! replaces the older direct-URL approach that YouTube has been deprecating
//! across all clients. This crate implements the full SABR flow:
//!
//! 1. InnerTube `/player` call to get streaming metadata
//! 2. Format selection (best audio, lowest video for discard trick)
//! 3. Protobuf request building (VideoPlaybackAbrRequest)
//! 4. UMP binary response parsing
//! 5. Audio segment extraction and streaming
//!
//! ## Usage
//!
//! ```rust,no_run
//! use sabr_rs::stream_audio;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//! let client = reqwest::Client::new();
//! let (info, mut rx) = stream_audio(&client, "dQw4w9WgXcQ").await?;
//!
//! println!("Title: {}", info.title);
//! println!("Duration: {}s", info.duration_secs);
//! println!("Format: {}", info.mime_type);
//!
//! while let Some(chunk) = rx.recv().await {
//!     // chunk is bytes::Bytes containing audio data
//!     // First chunk is the WebM init segment (container headers)
//!     // Subsequent chunks are audio segments in order
//! }
//! # Ok(())
//! # }
//! ```

pub mod ump;
pub mod stream;

pub(crate) mod proto {
    pub mod misc {
        include!(concat!(env!("OUT_DIR"), "/misc.rs"));
    }
    pub mod vs {
        include!(concat!(env!("OUT_DIR"), "/video_streaming.rs"));
    }
}

// Re-export the main public API
pub use stream::{stream_audio, SabrStreamInfo, SabrError};
pub use ump::{UmpParser, UmpPart};
