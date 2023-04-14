use crate::error::CAResult;
use core::mem::size_of;
use coreaudio_sys::{
    kAudioDevicePropertyMute, kAudioDevicePropertyScopeOutput, kAudioDevicePropertyVolumeScalar,
    kAudioHardwareNoError, kAudioHardwarePropertyDefaultOutputDevice,
    kAudioObjectPropertyElementMaster, kAudioObjectPropertyScopeGlobal, AudioDeviceID,
    AudioObjectGetPropertyData, AudioObjectHasProperty, AudioObjectPropertyAddress,
    AudioObjectSetPropertyData, OSStatus, UInt32,
};
use std::{
    ffi::c_void,
    ptr::{addr_of, null},
};

#[derive(Debug, Clone, Copy)]
pub enum Property {
    Volume,
    Mute,
    GetDefaultOutputDevice,
    Custom(AudioObjectPropertyAddress),
}

pub fn get_property<T: Default>(device_id: AudioDeviceID, property: Property) -> CAResult<T> {
    let result_container = T::default();
    let ptr = addr_of!(result_container) as *mut c_void;
    let mut data_size = size_of::<T>() as UInt32;

    let status = unsafe {
        AudioObjectGetPropertyData(device_id, &property.into(), 0, null(), &mut data_size, ptr)
    } as u32;

    if status != kAudioHardwareNoError {
        return Err(status as OSStatus);
    }
    Ok(result_container)
}

pub fn set_property<T>(device_id: AudioDeviceID, property: Property, value: &T) -> CAResult<()> {
    let ptr = addr_of!(*value) as *mut c_void;
    let data_size = size_of::<T>() as UInt32;

    let status = unsafe {
        AudioObjectSetPropertyData(device_id, &property.into(), 0, null(), data_size, ptr)
    } as u32;

    if status != kAudioHardwareNoError {
        return Err(status as OSStatus);
    }
    Ok(())
}

pub fn has_property(device_id: AudioDeviceID, property: Property) -> bool {
    let ret = unsafe { AudioObjectHasProperty(device_id, &property.into()) };
    ret != 0
}

impl From<Property> for AudioObjectPropertyAddress {
    fn from(value: Property) -> Self {
        match value {
            Property::Volume => Self {
                mSelector: kAudioDevicePropertyVolumeScalar,
                mScope: kAudioDevicePropertyScopeOutput,
                mElement: 0,
            },
            Property::Mute => Self {
                mSelector: kAudioDevicePropertyMute,
                mScope: kAudioDevicePropertyScopeOutput,
                mElement: 0,
            },
            Property::GetDefaultOutputDevice => Self {
                mSelector: kAudioHardwarePropertyDefaultOutputDevice,
                mScope: kAudioObjectPropertyScopeGlobal,
                mElement: kAudioObjectPropertyElementMaster,
            },
            Property::Custom(addr) => addr,
        }
    }
}

impl From<AudioObjectPropertyAddress> for Property {
    fn from(value: AudioObjectPropertyAddress) -> Self {
        Self::Custom(value)
    }
}
