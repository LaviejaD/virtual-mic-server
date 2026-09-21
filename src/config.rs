//! Configuración y utilidades generales.

use anyhow::{Context, Result};
use local_ip_address::local_ip;
use std::net::IpAddr;

/// Parámetros de audio esperados.
pub const SAMPLE_RATE: u32 = 44100;
pub const CHANNELS: u8 = 1;
/// Formato PCM: signed 16-bit little-endian.
pub const PCM_FORMAT: &str = "s16le";
/// Dispositivo de salida de PulseAudio.
pub const PA_DEVICE: &str = "only-virtual-mic-sink";

/// Obtiene la dirección IP local (LAN) del sistema.
pub fn get_lan_ip() -> Result<IpAddr> {
    local_ip().context("No se pudo determinar la IP local")
}
