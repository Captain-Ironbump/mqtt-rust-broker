mod models;

use futures::SinkExt;
use models::{broker::Broker, mqtt_types::{BrokerCommand, MqttPacketDispatcher, MqttPacketType}, packets::connect::Connect, packets::publish::Publish};

use tokio::{net::TcpListener, sync::{mpsc, oneshot}};
use tokio::spawn;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use futures_util::StreamExt;
use std::{ops::Deref, sync::{mpsc::Sender, Arc, Mutex}};

use log::{info, warn, error};
use env_logger;

const SERVER_ADDR: &str = "127.0.0.1";
const PORT: &str = "1883";


#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("logger initiated");
    let dispatcher = Arc::new(MqttPacketDispatcher::new().expect("Failed to create dispatcher")); 
    let listener = TcpListener::bind(format!("{}:{}", SERVER_ADDR, PORT)).await?;
    info!("WebSocket server listening on ws://{}:{}", SERVER_ADDR, PORT);

    let broker = Arc::new(Mutex::new(Broker::new()));

    while let Ok((stream, _)) = listener.accept().await {
        info!("New client connected: {:?}", stream.peer_addr());
        let dispatcher_clone = Arc::clone(&dispatcher);
        let broker_clone = Arc::clone(&broker);
        spawn(async move {
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    info!("WebSocket connecion established");
                    connection_handler(ws_stream, dispatcher_clone, broker_clone).await;
                }
                Err(e) => {
                    error!("Failed to upgrade TCP connection to WebSocket: {}", e);
                }
            }
        });
    }
    drop(listener);
    Ok(())
}


async fn connection_handler(ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, dispatcher: Arc<MqttPacketDispatcher>, broker: Arc<Mutex<Broker>>) {
    let (mut sender, mut receiver) = ws_stream.split(); // Split the stream
    info!("sender: [{:?}]; receiver: [{:?}]", sender, receiver);
    while let Some(message) = receiver.next().await {
        info!("Message: [{:?}]", message);
        match message {
            Ok(Message::Binary(data)) => {
                info!("We go here");
                let message_type = data[0] >> 4;  // Extract message type from the first byte
                let message_length = data[1];     // Extract message length from the second byte
                info!(
                    "Received WebSocket message of type {} and length {}",
                    message_type, message_length
                );
                let function = dispatcher.deref().handlers.get(&MqttPacketType::from_u8(message_type).unwrap()).unwrap();
                 
                // if let Ok(mut broker_guard) = broker.try_lock() {
                //     function(&data, &mut *broker_guard);
                // } else {
                //     error!("Failed to acquire lock on broker: it's already in use.");
                // }

                let packet = if let Ok(mut broker_guard) = broker.try_lock() {
                    let packet = function(&data, &mut *broker_guard);
                    drop(broker_guard);
                    Some(packet)
                } else {
                    error!("Failed to acquire lock on broker: it's already in use.");
                    None
                };

                if let Some(ref packet_data) = packet {
                    if packet_data.len() == 0 {
                        error!("Not a real packet data, no sending");
                        continue;
                    }
                    info!("packet_data: [{:?}]", packet_data);
                    if sender.send(Message::Binary(packet_data.to_vec())).await.is_err() {
                        error!("Failed to send packet of type: {:?}", packet_data[0] >> 4)
                    } else {
                        info!("Respoonded to Packet type: {:?}", message_type)
                    }
                }

                
                
                
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
                error!("Received text message, but expected binary data.");
            }
            Ok(Message::Close(_)) => {
                warn!("Received close frame from client, closing connection.");
                break;
            }
            Ok(_) => {
                error!("Received unsupported message type.");
            }
            Err(e) => {
                error!("WebSocket connection error: {:?}", e);
                break;
            }
        }
    }

    error!("Client disconnected.");
}

fn parse_packet(data: &Vec<u8>, ws_sender: Sender<Message>) -> Result<BrokerCommand, String> {
    if data.is_empty() {
        return Err("Empty data".to_string());
    }

    let message_type = data[0] >> 4;  // Extract message type from the first byte
    let message_length = data[1];     // Extract message length from the second byte
    info!(
        "Received WebSocket message of type {} and length {}",
        message_type, message_length
    );
    match message_type {
        1 => {
            // CONNECT message
            let connect_packet = Connect::from_bytes(data.clone());
            
            let (tx, rx) = oneshot::channel();
            Ok(BrokerCommand::Connect{
                packet: connect_packet,
                ws_sender: ws_sender.clone(),
                responder: tx,
            })
        }
        2 => {
            let (tx, rx) = oneshot::channel();
            // CONNACK message
            Ok(BrokerCommand::ConnAck{
                responder: tx,
            })
        },
        3 => {
            let (tx, rx) = oneshot::channel();
            // PUBLISH message
            let publish_packet = Publish::from_bytes(data.clone());

            Ok(BrokerCommand::Publish{
                packet: publish_packet,
                responder: tx,
            })
        },
        _ => {
            Err("Unknown message type".to_string())
        }
    }
}



// https://docs.solace.com/API/MQTT-311-Prtl-Conformance-Spec/MQTT%20Control%20Packets.htm
