//! Manejo de conexiones WebSocket.
use crate::arguments;
use crate::audio::spawn_pacat;
use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use hmac::KeyInit;
use log::{error, info, warn};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::{Duration, timeout};
use tokio_tungstenite::tungstenite::Message;

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).unwrap();
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// Maneja una conexión WebSocket individual.
///
/// Recibe mensajes binarios y los escribe en el stdin del proceso `pacat`.
/// Si la conexión se cierra o hay un error, termina el proceso hijo.
pub async fn handle_connection<S>(stream: S, peer: SocketAddr, pin: Arc<String>) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .context("Error durante el handshake WebSocket")?;
    info!("Nueva conexión desde: {}", peer);
    // Aceptar el handshake WebSocket.

    // Dividir el stream WebSocket en sink y stream.
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    if arguments::exist("-p".to_string()) {
        // 1) Enviar challenge
        let challenge: [u8; 32] = rand::random();
        ws_sender.send(Message::Binary(challenge.to_vec())).await?;

        // 2) Esperar respuesta del cliente
        const MAX_ATTEMPTS: usize = 5;
        const PAIR_TIMEOUT: Duration = Duration::from_secs(5);
        let client_resp = match timeout(PAIR_TIMEOUT, async {
            let mut attempts = 0;
            loop {
                if attempts >= MAX_ATTEMPTS {
                    return None;
                }
                attempts += 1;
                match ws_receiver.next().await {
                    Some(Ok(Message::Binary(data))) if data.len() == 32 => return Some(data),
                    Some(Ok(_)) => continue,
                    _ => return None,
                }
            }
        })
        .await
        {
            Ok(Some(data)) => data,
            _ => {
                warn!("Handshake fallido o timeout con {}", peer);
                let _ = ws_sender.send(Message::Close(None)).await;
                return Ok(());
            }
        };

        // 3) Verificar
        let expected = hmac_sha256(pin.as_bytes(), &challenge);
        if client_resp != expected {
            warn!("PIN incorrecto desde {}", peer);
            let _ = ws_sender.send(Message::Close(None)).await;

            return Ok(());
        }

        // 4) Responder al cliente
        let mut server_data = challenge.to_vec();
        server_data.extend_from_slice(b"server");
        let server_resp = hmac_sha256(pin.as_bytes(), &server_data);
        ws_sender.send(Message::Binary(server_resp)).await?;

        info!("Cliente {} autenticado", peer);
    }
    // Lanzar pacat para esta conexión.
    let (mut child, mut child_stdin) = spawn_pacat().await?;
    info!("Proceso pacat lanzado (PID: {})", child.id().unwrap_or(0));

    // Resultado del bucle: si hay error, se captura y se devuelve.
    let result = async {
        // Procesar mensajes entrantes.
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    // Escribir datos al stdin de pacat.
                    if let Err(e) = child_stdin.write_all(&data).await {
                        error!("Error escribiendo a pacat: {}", e);
                        return Err(e).context("Error escribiendo a pacat");
                    }
                }
                Ok(Message::Close(_)) | Err(_) => {
                    info!("Conexión cerrada por el cliente o error de red");
                    break;
                }
                Ok(Message::Text(text)) => {
                    // No esperamos texto, pero lo registramos.
                    warn!("Mensaje de texto inesperado: {:?}", text);
                }
                Ok(Message::Ping(payload)) => {
                    // Responder a los pings para mantener viva la conexión.
                    if let Err(e) = ws_sender.send(Message::Pong(payload)).await {
                        error!("Error enviando pong: {}", e);
                        break;
                    }
                }
                Ok(Message::Pong(_)) | Ok(Message::Frame(_)) => {
                    // Ignorar pongs y frames crudos.
                }
            }
        }
        Ok(())
    }
    .await;

    // Independientemente del resultado, cerrar stdin y matar el proceso pacat.
    info!("Cerrando stdin de pacat y terminando el proceso");
    drop(child_stdin); // Cierra el pipe.
    let _ = child.kill().await; // Intenta matar si aún está vivo.
    let _ = child.wait().await; // Espera a que termine.

    // Si el bucle terminó con error, propagarlo.
    result?;

    info!("Conexión con {} finalizada correctamente", peer);
    Ok(())
}
