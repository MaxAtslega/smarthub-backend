use std::net::SocketAddr;

use log::{error, info};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::{Error, Result};

use crate::config::WebSocketConf;
use crate::enums::system_command::SystemCommand;
use crate::handlers::connection_handler::handle_connection;
use crate::models::notification_response::NotificationResponse;

pub async fn init(web_socket_conf: &WebSocketConf, mut tx: tokio::sync::broadcast::Sender<NotificationResponse>, rx: Receiver<NotificationResponse>, tx_dbus: Sender<SystemCommand>) -> Result<(), Error> {
    let address = format!("{}:{}", web_socket_conf.address, web_socket_conf.port);
    let try_socket = TcpListener::bind(&address).await;

    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", address);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);

        let rx_clone = rx.resubscribe();
        let tx_dbus_clone = tx_dbus.clone();
        let tx_clone = tx.clone();

        tokio::spawn(accept_connection(peer, stream, tx_clone, rx_clone, tx_dbus_clone));
    }

    Ok(())
}


async fn accept_connection(peer: SocketAddr, stream: TcpStream, tx: tokio::sync::broadcast::Sender<NotificationResponse>, rx: Receiver<NotificationResponse>, tx_dbus: Sender<SystemCommand>) {
    if let Err(e) = handle_connection(peer, stream, tx, rx, tx_dbus).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}