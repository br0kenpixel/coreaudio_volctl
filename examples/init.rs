use coreaudio_volctl::AudioOutputDevice;

fn main() {
    let dev = AudioOutputDevice::get_default().unwrap();
    println!("{dev:?}");

    println!("Volume: {}%", dev.get_volume().unwrap());
    println!("Muted: {}", dev.is_muted().unwrap());
}
