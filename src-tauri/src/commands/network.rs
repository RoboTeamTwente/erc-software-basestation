use tauri::State;

use crate::network::service::UdpService;
use crate::network::sender;
use crate::proto::packets::Ping;

#[tauri::command]
pub async fn send_ping_cmd(
    state: State<'_, UdpService>,
) -> Result<(), String> {
    println!("Called ping command");

    let socket = state.socket();

    println!("Socket gotten");

    let packet = Ping {
        id: 1,
        payload: "hello".into(),
    };

    println!("Packet encoded");

    sender::send_ping(&socket, "127.0.0.1:9000", packet)
        .await
        .map_err(|e| e.to_string())?;

    println!("Packet sent");
    Ok(())
}