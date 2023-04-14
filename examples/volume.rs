use coreaudio_volctl::AudioOutputDevice;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let dev = AudioOutputDevice::get_default().unwrap();
    println!("{dev:?}");
    let last_vol = dev.get_volume().unwrap();

    println!("Setting volume to 25% for 1 second...");
    dev.set_volume(25).unwrap();

    sleep(Duration::from_secs(1));

    println!("Resetting back to {last_vol}%...");
    dev.set_volume(last_vol).unwrap();
}
