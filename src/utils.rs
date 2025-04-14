use windows::Win32::{Foundation::*, NetworkManagement::WiFi::*};

pub fn check_signal_strength(network: &WLAN_AVAILABLE_NETWORK) -> u32 {
    let strength = network.wlanSignalQuality;
    let bars = match strength {
        80..=100 => 4,
        60..=79 => 3,
        40..=59 => 2,
        20..=39 => 1,
        _ => 0,
    };

    bars
}

pub fn check_win32(result: u32) -> Result<(), WIN32_ERROR> {
    if result == ERROR_SUCCESS.0 {
        Ok(())
    } else {
        Err(WIN32_ERROR(result))
    }
}