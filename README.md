
# virtual-mic-server

Servidor de micrófono virtual para distribuciones Linux. El objetivo del proyecto es ofrecer una alternativa para usar el micrófono del teléfono como micrófono inalámbrico.

## Prerrequisitos

Tu sistema debe usar **PulseAudio** o **PipeWire** (con `pipewire-pulse`). Verifica que `pacat` esté disponible:

```bash
pacat --version
```

Si no lo está, instálalo según tu distro:

### Ubuntu, Debian, Linux Mint y derivados

```bash
sudo apt update && sudo apt install pulseaudio-utils
```

### Fedora, RHEL y CentOS

```bash
sudo dnf install pulseaudio-utils
```

### Arch Linux y Manjaro

Si usas PipeWire (recomendado):

```bash
sudo pacman -S pipewire-pulse
```

Si usas PulseAudio clásico:

```bash
sudo pacman -S pulseaudio pulseaudio-utils
```

## Cómo usar el proyecto

Primero dale permisos de ejecución al binario:

```bash
chmod +x virtual-mic-server
```

Puedes descargarlo [aquí](https://github.com/LaviejaD/virtual-mic-server/releases/latest).

### Ejecutar el binario

```bash
./virtual-mic-server
```

### Ejecutar con Cargo

Necesitas tener instalado [Rust](https://rust-lang.org/).

```bash
cargo run --release
```

### Argumentos

Si quieres restringir la conexión a ciertos dispositivos, usa `-p`:

```bash
./virtual-mic-server -p myPassword
```

## Cliente móvil

Puedes encontrar el APK del cliente [aquí](https://github.com/LaviejaD/mic-client).

## Configurar el micrófono virtual

### Temporal

Crea el dispositivo manualmente. Se eliminará al reiniciar:

```bash
pactl load-module module-null-sink sink_name=only-virtual-mic-sink sink_properties=device.description=Virtual-Mic-Sink
pactl load-module module-remap-source master=only-virtual-mic-sink.monitor source_name=only-virtual-mic-source source_properties=device.description=Virtual-Mic
```

### Permanente (PipeWire)

Crea el directorio si no existe:

```bash
mkdir -p ~/.config/pipewire/pipewire-pulse.conf.d/
```

Luego crea `~/.config/pipewire/pipewire-pulse.conf.d/virtual-mic.conf` con:

```
pulse.cmd = [
    { cmd = "load-module" args = "module-null-sink sink_name=only-virtual-mic-sink sink_properties=device.description=Virtual-Mic-Sink" flags = [ ] },
    { cmd = "load-module" args = "module-remap-source master=only-virtual-mic-sink.monitor source_name=only-virtual-mic-source source_properties=device.description=Virtual-Mic" flags = [ ] }
]
```

Reinicia PipeWire para aplicar los cambios:

```bash
systemctl --user restart pipewire pipewire-pulse
```

> **Nota**: en PulseAudio clásico, la persistencia se hace editando `~/.config/pulse/default.pa` con líneas `load-module ...`.
