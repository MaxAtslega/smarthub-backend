use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use serde_derive::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::Receiver;
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, Result},
};

use crate::config::WebSocketConf;
use crate::models::notification_response::NotificationResponse;
use crate::handlers::connection_handler::handle_connection;

pub async fn init(web_socket_conf: &WebSocketConf, rx: Receiver<NotificationResponse>) -> Result<(), Error> {
    let address = format!("{}:{}", web_socket_conf.address, web_socket_conf.port);
    let try_socket = TcpListener::bind(&address).await;

    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", address);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);

        let rx_clone = rx.resubscribe();
        tokio::spawn(accept_connection(peer, stream, rx_clone));
    }

    Ok(())
}


async fn accept_connection(peer: SocketAddr, stream: TcpStream, rx: Receiver<NotificationResponse>) {
    if let Err(e) = handle_connection(peer, stream, rx).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}