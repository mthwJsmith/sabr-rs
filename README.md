# sabr-rs

Rust implementation of YouTube's SABR (Server Adaptive Bit Rate) streaming protocol.

Give it a YouTube video ID, get back a stream of audio bytes (WebM/Opus).

## What is SABR?

SABR is YouTube's native adaptive streaming protocol. It's what the official YouTube app uses under the hood. YouTube has been migrating all clients to SABR and deprecating the older direct-URL approach, so this is the forward-looking way to get audio/video from YouTube.

The protocol works like this:

1. Make an InnerTube `/player` API call to get streaming metadata (formats, CDN URL, etc.)
2. Build a protobuf `VideoPlaybackAbrRequest` describing what you want
3. POST it to YouTube's CDN
4. Parse the response as UMP (Universal Media Protocol) binary frames
5. Extract audio segments from the UMP stream
6. Repeat, reporting what you've buffered so the server sends the next segments

## How it works

The crate handles the full SABR flow:

- **InnerTube /player call** with IOS client identity (falls back to ANDROID if needed). Gets the streaming URL, format list, ustreamer config, title, and duration.
- **Format selection**: picks the highest-bitrate Opus audio format. Also selects the lowest-bitrate video format for the "discard trick" (see below).
- **Video discard trick**: SABR requires both audio and video formats in every request, even if you only want audio. The trick is to report a `BufferedRange` with `i32::MAX` values for the video format, so the server thinks you already have all the video and never sends any. The actual audio-only flag (`enabledTrackTypesBitfield=1`) is also set.
- **UMP parser**: YouTube uses a custom binary framing format (not standard protobuf varint) for the response stream. The parser handles chunked delivery and extracts typed parts (media headers, media data, media end markers, next request policies, SABR context updates, redirects, errors, etc.).
- **Retry logic**: exponential backoff (500ms to 8s) on transient network errors, up to 5 retries.
- **Streaming output**: audio bytes are sent through a tokio mpsc channel as they arrive. The init segment (WebM container headers) comes first, then audio segments in order.

## Usage

```rust
use sabr_rs::stream_audio;

let client = reqwest::Client::new();
let (info, mut rx) = stream_audio(&client, "dQw4w9WgXcQ").await?;

println!("{} [{}s] ({})", info.title, info.duration_secs, info.mime_type);

while let Some(chunk) = rx.recv().await {
    // Write chunk to file, pipe to HTTP response, feed to audio player, etc.
}
```

The returned `SabrStreamInfo` contains:
- `title` - video title from YouTube
- `duration_secs` - duration in seconds
- `mime_type` - e.g. `audio/webm; codecs="opus"`

The channel yields `bytes::Bytes` chunks. First chunk is the WebM init segment, then audio data segments in order. When all segments are downloaded the channel closes.

## Proto files

The `proto/` directory contains protobuf definitions for SABR's wire format. These are compiled at build time via `prost-build`. The protos come from the [googlevideo](https://github.com/LuanRT/googlevideo) project by LuanRT.

## Credits

- **[LuanRT/googlevideo](https://github.com/LuanRT/googlevideo)** (MIT) - TypeScript SABR reference implementation and protobuf definitions. This crate's protocol logic is a Rust port of that work. The proto files in `proto/` are from that repo.
- **[nichobi/yt-sabr-shaka-demo](https://github.com/LuanRT/yt-sabr-shaka-demo)** - Minimal SABR demo that helped clarify the request/response flow.
- **[LuanRT/YouTube.js](https://github.com/LuanRT/YouTube.js)** - YouTube's InnerTube API reverse engineering.

## License

MIT
