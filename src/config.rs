//! Configuración y utilidades generales.
use anyhow::Result;
use dirs;
use local_ip_address::local_ip;
use log::error;
use std::net::IpAddr;
use std::path::PathBuf;

/// Parámetros de audio esperados.
pub const SAMPLE_RATE: u32 = 44100;
pub const CHANNELS: u8 = 1;
/// Formato PCM: signed 16-bit little-endian.
pub const PCM_FORMAT: &str = "s16le";
/// Dispositivo de salida de PulseAudio.
pub const PA_DEVICE: &str = "only-virtual-mic-sink";
/// puerto por defecto
pub const PREFERRED_PORT: u16 = 3000;
pub fn get_lan_ip() -> Result<IpAddr, ()> {
    match local_ip() {
        Ok(r) => Ok(r),
        Err(_) => {
            error!("Error en obtener la ip local.");
            return Err(());
        }
    }
}
/// Obtener directorio de configuraciones
pub fn get_config_dir() -> Option<PathBuf> {
    dirs::config_dir()
}
