mod models;

use models::mqtt_types::{MqttPacketType, MqttPacketDispatcher};

use tokio::net::TcpListener;
use tokio::spawn;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use futures_util::StreamExt;
use std::{ops::Deref, sync::Arc};
  
const SERVER_ADDR: &str = "127.0.0.1";
const PORT: &str = "1883";


#[tokio::main]
async fn main() -> std::io::Result<()> {
    let dispatcher = Arc::new(MqttPacketDispatcher::new().expect("Failed to create dispatcher")); 
    let listener = TcpListener::bind(format!("{}:{}", SERVER_ADDR, PORT)).await?;
    println!("WebSocket server listening on ws://{}:{}", SERVER_ADDR, PORT);
    while let Ok((stream, _)) = listener.accept().await {
        println!("New client connected: {:?}", stream.peer_addr());
        let dispatcher_clone = Arc::clone(&dispatcher);
        spawn(async move {
            if let Ok(ws_stream) = accept_async(stream).await {
                connection_handler(ws_stream, dispatcher_clone).await;
            } else {
                eprintln!("Failed to upgrade TCP connection to WebSocket")
            }
        });
    }
    drop(listener);
    Ok(())
}


async fn connection_handler(ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, dispatcher: Arc<MqttPacketDispatcher>) {
    let (mut sender, mut receiver) = ws_stream.split(); // Split the stream
    while let Some(message) = receiver.next().await {
        match message {
            Ok(Message::Binary(data)) => {
                let message_type = data[0] >> 4;  // Extract message type from the first byte
                let message_length = data[1];     // Extract message length from the second byte
                println!(
                    "Received WebSocket message of type {} and length {}",
                    message_type, message_length
                );
                let function = dispatcher.deref().handlers.get(&MqttPacketType::from_u8(message_type).unwrap()).unwrap();
                function(&mut sender, &data);
                
                
                // match message_type {
                //     1 => {
                //         // CONNECT message
                //         let connack_packet: Vec<u8> = vec![
                //             0x20, // CONNACK Packet type
                //             0x02, // Remaining length
                //             0x00, // Connection accepted
                //             0x00, // Connection accepted
                //         ];

                //         // Send CONNACK response as a WebSocket binary message
                //         if ws_stream.send(Message::Binary(connack_packet)).await.is_err() {
                //             eprintln!("Failed to send CONNACK packet");
                //         } else {
                //             println!("Responded to CONNECT");
                //         }
                //     }
                //     t => {
                //         eprintln!("Unknown type of message: {}", t);
                //     }
                // }
            }
            Ok(Message::Text(_)) => {
                eprintln!("Received text message, but expected binary data.");
            }
            Ok(_) => {
                eprintln!("Received unsupported message type.");
            }
            Err(e) => {
                eprintln!("WebSocket connection error: {:?}", e);
                break;
            }
        }
    }

    eprintln!("Client disconnected.");
}



// https://docs.solace.com/API/MQTT-311-Prtl-Conformance-Spec/MQTT%20Control%20Packets.htm