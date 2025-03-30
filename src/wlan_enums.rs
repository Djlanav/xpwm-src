use windows::Win32::NetworkManagement::WiFi::*;

pub enum WlanInterfaceState {
    Connected,
    Disconnected,
    Associating,
    Authenticating,
    Discovering,
    Unavailable,
}

pub fn convert_wlan_interface_state(state: WLAN_INTERFACE_STATE) -> WlanInterfaceState {
    let state_enum = match state {
        wlan_interface_state_connected => WlanInterfaceState::Connected,
        wlan_interface_state_disconnected => WlanInterfaceState::Disconnected,
        wlan_interface_state_associating => WlanInterfaceState::Associating,
        wlan_interface_state_authenticating => WlanInterfaceState::Authenticating,
        wlan_interface_state_discovering => WlanInterfaceState::Discovering,
        _ => WlanInterfaceState::Unavailable,
    };

    state_enum
}