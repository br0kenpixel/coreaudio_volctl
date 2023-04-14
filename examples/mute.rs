use coreaudio_volctl::AudioOutputDevice;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let dev = AudioOutputDevice::get_default().unwrap();
    println!("{dev:?}");

    println!("Muting for 1 second...");
    dev.set_mute(true).unwrap();

    sleep(Duration::from_secs(1));

    println!("Unmuting...");
    dev.set_mute(false).unwrap();
}
