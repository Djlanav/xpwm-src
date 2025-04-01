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

#[derive(Debug, Clone, GodotConvert, Var, Export)]
#[godot(via = GString)]
pub enum ConnectionNotifcation {
    ConnectionStart,
    //AuthenticateComplete,
    ConnectionComplete,
    ConnectionAttemptFail,
    Disconnected,
    Unknown
}

#[allow(non_upper_case_globals)]
pub fn convert_connection_notification(code: WLAN_NOTIFICATION_ACM) -> ConnectionNotifcation {
    let notif = match code {
        wlan_notification_acm_connection_start => ConnectionNotifcation::ConnectionStart,
        wlan_notification_acm_connection_complete => ConnectionNotifcation::ConnectionComplete,
        wlan_notification_acm_disconnected => ConnectionNotifcation::Disconnected,
        wlan_notification_acm_connection_attempt_fail => ConnectionNotifcation::ConnectionAttemptFail,

        _ => ConnectionNotifcation::Unknown
    };

    notif
}

#[allow(non_upper_case_globals)]
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