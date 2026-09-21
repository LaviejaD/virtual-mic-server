//! Lanzamiento y gestión del proceso `pacat`.

use crate::config::{CHANNELS, PA_DEVICE, PCM_FORMAT, SAMPLE_RATE};
use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::process::{Child, ChildStdin, Command};

/// Lanza el proceso `pacat` para reproducción con los parámetros de audio definidos.
///
/// Devuelve el `Child` y su `stdin` para escribir los datos de audio.
pub async fn spawn_pacat() -> Result<(Child, ChildStdin)> {
    let mut child = Command::new("pacat")
        .arg("--playback")
        .arg(format!("--format={}", PCM_FORMAT))
        .arg(format!("--rate={}", SAMPLE_RATE))
        .arg(format!("--channels={}", CHANNELS))
        .arg("--device")
        .arg(PA_DEVICE.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("No se pudo iniciar pacat. ¿Está instalado PulseAudio y pacat?")?;

    let stdin = child
        .stdin
        .take()
        .context("No se pudo obtener stdin de pacat")?;

    Ok((child, stdin))
}
