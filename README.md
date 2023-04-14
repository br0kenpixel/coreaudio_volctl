# CoreAudio - Volume Control
A simple library for setting/getting mute state and volume level on macOS.

### Example
```rust
use coreaudio_volctl::AudioOutputDevice;

fn main() {
    let dev = AudioOutputDevice::get_default().unwrap();
    println!("{dev:?}");

    println!("Volume: {}%", dev.get_volume().unwrap());
    println!("Muted: {}", dev.is_muted().unwrap());
}
```

### Compatible OSs
Currently it's been tested only on macOS Monterey 12.6.5 (21G531). However, it should work on older versions down to Big Sur.
It may __not__ compile on macOS Catalina and older due to some API changes in CoreAudio.