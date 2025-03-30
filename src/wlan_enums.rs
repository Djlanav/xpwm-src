use godot::prelude::*;
use windows::Win32::NetworkManagement::WiFi::*;

#[derive(Clone, GodotConvert, Var, Export)]
#[godot(via = GString)]
pub enum NetworkSecurity {
    Open,
    WPA,
    WPA2,
    WPAPSK,
    WPA2PSK,
    Unknown
}

impl Default for NetworkSecurity {
    fn default() -> NetworkSecurity {
        NetworkSecurity::Open
    }
}

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

pub fn check_wlan_interface_state<F>(
    state: &WlanInterfaceState,
    mut closure: F) -> bool
where
    F: FnMut() -> bool
{
    match state {
        WlanInterfaceState::Connected => closure(),
        WlanInterfaceState::Disconnected => closure(),
        WlanInterfaceState::Associating => closure(),
        WlanInterfaceState::Authenticating => closure(),
        WlanInterfaceState::Discovering => closure(),
        WlanInterfaceState::Unavailable => false
    }
}