#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_lossless
)]

use coreaudio_sys::{
    kAudioObjectSystemObject, AudioDeviceID, AudioObjectPropertyAddress, Float32, UInt32,
};

/// Error type
pub mod error;
use error::CAResult;
mod safe_wrappers;
use safe_wrappers::{get_property, has_property, set_property, Property};

const CHANNEL_CHECK_FAILS: usize = 3;

/// Audio output device controller
///
/// # Note
/// Changing the default audio output device __after__ an instance is created will not affect it!
/// You'll need to get a new instance using [`get_default()`](Self::get_default).
#[derive(Debug)]
pub struct AudioOutputDevice {
    device_id: AudioDeviceID,
    valid_channels: Vec<u32>,
}

impl AudioOutputDevice {
    /// Gets the currently set default audio output device on your system.
    ///
    /// # Errors
    /// This method may fail if [`AudioObjectGetPropertyData`](coreaudio_sys::AudioObjectGetPropertyData) fails.
    /// # Example
    /// ```rust
    /// use coreaudio_volctl::AudioOutputDevice;
    ///
    /// fn main() {
    ///     let device = AudioOutputDevice::get_default().unwrap();
    ///     // ...
    /// }
    /// ```
    pub fn get_default() -> CAResult<Self> {
        let device_id = Self::get_default_device_id()?;
        let valid_channels = Self::get_valid_channels(device_id);

        Ok(Self {
            device_id,
            valid_channels,
        })
    }

    /// Gets the currently set volume of the device.
    /// The returned volume is in percents.
    ///
    /// # Errors
    /// This method may fail if [`AudioObjectGetPropertyData`](coreaudio_sys::AudioObjectGetPropertyData) fails.
    ///
    /// # Notes
    /// Due to the fact that internally the volume level is requested in scalar units, and later converted to [`u8`](u8), there is
    /// a small precision loss.
    /// Since the volume values have to be requested from each channel individually, their average is computed and returned.
    /// # Example
    /// ```rust
    /// use coreaudio_volctl::AudioOutputDevice;
    ///
    /// fn main() {
    ///     let device = AudioOutputDevice::get_default().unwrap();
    ///     let volume: u8 = device.get_volume().unwrap();
    ///     
    ///     println!("The current volume level is {volume}%");
    /// }
    /// ```
    pub fn get_volume(&self) -> CAResult<u8> {
        let mut address: AudioObjectPropertyAddress = Property::Volume.into();
        let mut values = Vec::new();

        for channel in &self.valid_channels {
            address.mElement = *channel;
            values.push(get_property::<Float32>(self.device_id, address.into())? * 100.0);
        }
        let avg = values.iter().sum::<f32>() / values.len() as f32;

        Ok(avg as u8)
    }

    /// Gets whether the device is muted or not.
    ///
    /// # Errors
    /// This method may fail if [`AudioObjectGetPropertyData`](coreaudio_sys::AudioObjectGetPropertyData) fails.
    /// # Example
    /// ```rust
    /// use coreaudio_volctl::AudioOutputDevice;
    ///
    /// fn main() {
    ///     let device = AudioOutputDevice::get_default().unwrap();
    ///     let muted = device.is_muted().unwrap();
    ///     
    ///     if muted {
    ///         println!("The device is muted.");
    ///     } else {
    ///         println!("The device is unmuted.");
    ///     }
    /// }
    /// ```
    pub fn is_muted(&self) -> CAResult<bool> {
        Ok(get_property::<i32>(self.device_id, Property::Mute)? != 0)
    }

    /// Sets the mute status of the device.
    ///
    /// # Errors
    /// This method may fail if [`AudioObjectGetPropertyData`](coreaudio_sys::AudioObjectGetPropertyData) fails.
    /// # Example
    /// ```rust
    /// use coreaudio_volctl::AudioOutputDevice;
    /// use std::time::Duration;
    /// use std::thread::sleep;
    ///
    /// fn main() {
    ///     let device = AudioOutputDevice::get_default().unwrap();
    ///     
    ///     device.set_mute(true).unwrap();
    ///     println!("The device is now muted");
    ///     
    ///     sleep(Duration::from_seconds(2));
    ///
    ///     device.set_mute(false).unwrap();
    ///     println!("The device is now unmuted");
    /// }
    /// ```
    pub fn set_mute(&self, mute: bool) -> CAResult<()> {
        let mut address: AudioObjectPropertyAddress = Property::Mute.into();
        let mute = mute as UInt32;
        let mut results = Vec::new();

        for channel in &self.valid_channels {
            address.mElement = *channel;
            results.push(set_property(self.device_id, address.into(), &mute));
        }

        if results.iter().any(Result::is_err) {
            address.mElement = 0;
            return set_property(self.device_id, address.into(), &mute);
        }

        Ok(())
    }

    /// Sets a volume percentage.
    /// The `vol` argument should be a value between `0` and `100`.
    ///
    /// # Errors
    /// This method may fail if [`AudioObjectGetPropertyData`](coreaudio_sys::AudioObjectGetPropertyData) fails.
    /// # Notes
    /// `vol` is [`clamp()`](u8::clamp)ed, so it's safe to send values > `100`.
    /// # Example
    /// ```rust
    /// use coreaudio_volctl::AudioOutputDevice;
    ///
    /// fn main() {
    ///     let device = AudioOutputDevice::get_default().unwrap();
    ///     
    ///     device.set_volume(25).unwrap(); // Set volume to 25%
    ///     device.set_volume(50).unwrap(); // Set volume to 50%
    ///     device.set_volume(100).unwrap(); // Set volume to 100%
    /// }
    /// ```
    pub fn set_volume(&self, vol: u8) -> CAResult<()> {
        let vol = vol.clamp(0, 100) as Float32 / 100.0;
        let mut address: AudioObjectPropertyAddress = Property::Volume.into();

        for channel in &self.valid_channels {
            address.mElement = *channel;
            set_property(self.device_id, address.into(), &vol)?;
        }

        Ok(())
    }

    /// Gets whether the device controlled by this instance is the default output device on the system.
    /// You may create a new instance of [`AudioOutputDevice`](Self) if this returns `false`. This can be useful
    /// for projects there you need to detect if the default output device was changed.
    ///
    /// # Errors
    /// This method may fail if [`AudioObjectGetPropertyData`](coreaudio_sys::AudioObjectGetPropertyData) fails.
    /// # Example
    /// ```rust
    /// use coreaudio_volctl::AudioOutputDevice;
    ///
    /// fn main() {
    ///     let mut device = AudioOutputDevice::get_default().unwrap();
    ///     
    ///     loop {
    ///         if !device.is_default() {
    ///             println!("You changed the default output device!");
    ///         }
    ///         // Get a new instance so we can control the new device.
    ///         device = AudioOutputDevice::get_default().unwrap();
    ///     }
    /// }
    /// ```
    pub fn is_default(&self) -> CAResult<bool> {
        Ok(Self::get_default_device_id()? == self.device_id)
    }

    fn get_default_device_id() -> CAResult<AudioDeviceID> {
        get_property(kAudioObjectSystemObject, Property::GetDefaultOutputDevice)
    }

    fn get_valid_channels(id: AudioDeviceID) -> Vec<u32> {
        let mut result = Vec::new();
        let mut address: AudioObjectPropertyAddress = Property::Volume.into();
        let mut failures = 0;

        while failures < CHANNEL_CHECK_FAILS {
            if has_property(id, address.into()) {
                result.push(address.mElement);
            } else {
                failures += 1;
            }
            address.mElement += 1;
        }

        result
    }
}
