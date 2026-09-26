mod arguments;
mod audio;
mod config;
mod tls;
mod utils;
mod websocket;

use anyhow::{Context, Result};
use log::{error, info, warn};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use config::get_lan_ip;
use websocket::handle_connection;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // IP local.
    let ip = get_lan_ip()?;
    info!("IP local detectada: {}", ip);
    //pin
    let pin = std::sync::Arc::new(utils::generate_pin(6));

    // Configuración TLS (carga o genera certificado).
    let tls_config = tls::load_or_generate_tls_config(&ip.to_string())?;
    let tls_acceptor = TlsAcceptor::from(Arc::clone(&tls_config));

    // Enlazar.
    let preferred_port: u16 = 3000;
    let listener = match TcpListener::bind((ip, preferred_port)).await {
        Ok(listener) => {
            info!("Escuchando en {}:{}", ip, preferred_port);
            listener
        }
        Err(e) => {
            warn!(
                "No se pudo enlazar al puerto {}: {}. Intentando con puerto automático.",
                preferred_port, e
            );
            let listener = TcpListener::bind((ip, 0))
                .await
                .context("No se pudo enlazar a ningún puerto")?;
            let addr = listener.local_addr()?;
            info!("Escuchando en {}:{} (puerto automático)", ip, addr.port());
            listener
        }
    };

    let local_addr = listener.local_addr()?;

    println!("===========================================");
    println!(" Servidor WebSocket WSS (TLS) iniciado     ");
    println!(
        " Dirección: wss://{}:{}",
        local_addr.ip(),
        local_addr.port()
    );
    if arguments::exist("-p".to_string()) {
        println!("Ping: {}", pin)
    }
    println!("===========================================");
    loop {
        let (tcp_stream, peer) = listener.accept().await?;
        info!("Conexión TCP aceptada de {}", peer);

        let acceptor = tls_acceptor.clone();
        let pin = pin.clone();

        tokio::spawn(async move {
            // 1) Handshake TLS
            let tls_stream = match acceptor.accept(tcp_stream).await {
                Ok(s) => s,
                Err(e) => {
                    error!("Error en handshake TLS con {}: {}", peer, e);
                    return;
                }
            };
            info!("TLS establecido con {}", peer);

            // 2) Handshake WebSocket + lógica
            if let Err(e) = handle_connection(tls_stream, peer, pin).await {
                error!("Error manejando conexión de {}: {:?}", peer, e);
            }
        });
    }
}
