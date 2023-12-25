use std::net::SocketAddr;

use log::{error, info};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::{Error, Result};

use crate::app::DbusCommand;
use crate::config::WebSocketConf;
use crate::handlers::connection_handler::handle_connection;
use crate::models::notification_response::NotificationResponse;

pub async fn init(web_socket_conf: &WebSocketConf, rx: Receiver<NotificationResponse>, tx_dbus: Sender<DbusCommand>) -> Result<(), Error> {
    let address = format!("{}:{}", web_socket_conf.address, web_socket_conf.port);
    let try_socket = TcpListener::bind(&address).await;

    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", address);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);

        let rx_clone = rx.resubscribe();
        let tx_clone = tx_dbus.clone();
        accept_connection(peer, stream, rx_clone, tx_clone).await;
    }

    Ok(())
}


async fn accept_connection(peer: SocketAddr, stream: TcpStream, rx: Receiver<NotificationResponse>, tx_dbus: Sender<DbusCommand>) {
    if let Err(e) = handle_connection(peer, stream, rx, tx_dbus).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}