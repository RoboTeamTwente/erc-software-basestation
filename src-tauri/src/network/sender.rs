use tokio::net::UdpSocket;                                                                               
use prost::Message;
use crate::proto::packets::*;

/// Encode any `PbEnvelope` payload, log the raw bytes, and send over UDP.
pub async fn send_envelope(
    socket: &UdpSocket,
    addr: &str,
    envelope: PbEnvelope,
) -> anyhow::Result<()> {
    let mut payload = Vec::new();
    envelope.encode(&mut payload)?;

    // println!(
    //     "[UDP TX] {} bytes to {}: {}",
    //     payload.len(),
    //     addr,
    //     hex_dump(&payload)
    // );

    socket.send_to(&payload, addr).await?;
    Ok(())
}

/// Format a byte slice as "AA BB CC …" (hex), capped at 64 bytes to keep logs readable.
fn hex_dump(bytes: &[u8]) -> String {
    let preview = &bytes[..bytes.len().min(64)];
    let hex: Vec<String> = preview.iter().map(|b| format!("{b:02X}")).collect();
    let mut out = hex.join(" ");
    if bytes.len() > 64 {
        out.push_str(&format!(" … (+{} more bytes)", bytes.len() - 64));
    }
    out
}