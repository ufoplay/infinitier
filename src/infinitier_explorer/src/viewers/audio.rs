//! The audio output shared by the sound and movie players.

use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};

pub struct Audio {
    /// Output device handle; must outlive `sink`.
    _stream: MixerDeviceSink,
    pub sink: Player,
}

/// The default output device with a paused player, or `None` if the system
/// has no usable audio device.
pub fn init_audio() -> Option<Audio> {
    let mut device = DeviceSinkBuilder::open_default_sink().ok()?;
    // Avoid a noisy "device dropped while still streaming" log when the
    // viewer is replaced: the explorer flips between viewers regularly.
    device.log_on_drop(false);
    let sink = Player::connect_new(device.mixer());
    sink.pause(); // start paused; the user presses Play to begin
    Some(Audio {
        _stream: device,
        sink,
    })
}
