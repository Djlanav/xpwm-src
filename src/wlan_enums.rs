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

impl NetworkSecurity {
    pub fn convert_to_string(&self) -> String {
        match self {
            NetworkSecurity::Open => String::from("Open"),
            NetworkSecurity::WPA => String::from("WPA"),
            NetworkSecurity::WPA2 => String::from("WPA2"),
            NetworkSecurity::WPAPSK => String::from("WPAPSK"),
            NetworkSecurity::WPA2PSK => String::from("WPA2PSK"),
            NetworkSecurity::Unknown => String::from("Unknown"),
        }
    }
}

pub fn check_security(network: &WLAN_AVAILABLE_NETWORK) -> (bool, NetworkSecurity) {
    let is_secured = network.bSecurityEnabled.as_bool();

    let security_type = match network.dot11DefaultAuthAlgorithm {
        DOT11_AUTH_ALGO_80211_OPEN => NetworkSecurity::Open,
        DOT11_AUTH_ALGO_WPA => NetworkSecurity::WPA,
        DOT11_AUTH_ALGO_WPA_PSK => NetworkSecurity::WPAPSK,
        DOT11_AUTH_ALGO_RSNA => NetworkSecurity::WPA2,
        DOT11_AUTH_ALGO_RSNA_PSK => NetworkSecurity::WPA2PSK,
        _ => NetworkSecurity::Unknown
    };

    (is_secured, security_type)
}

#[derive(Clone, GodotConvert, Var, Export)]
#[godot(via = GString)]
pub enum EncryptionAlgorithm {
    AES,
    TKIP,
    None
}

impl EncryptionAlgorithm {
    pub fn convert_to_string(&self) -> String {
        match self {
            EncryptionAlgorithm::AES => String::from("AES"),
            EncryptionAlgorithm::TKIP => String::from("TKIP"),
            EncryptionAlgorithm::None => String::from("NONE"),
        }
    }
}

pub fn check_encryption(network: &WLAN_AVAILABLE_NETWORK) -> EncryptionAlgorithm     {
    let network_encryption = match network.dot11DefaultCipherAlgorithm {
        DOT11_CIPHER_ALGO_CCMP => EncryptionAlgorithm::AES,
        DOT11_CIPHER_ALGO_TKIP => EncryptionAlgorithm::TKIP,
        DOT11_CIPHER_ALGO_NONE => EncryptionAlgorithm::None,
        _ => EncryptionAlgorithm::None
    };

    network_encryption
}

#[derive(Debug, Clone, GodotConvert, Var, Export)]
#[godot(via = GString)]
pub enum ConnectionNotifcation {
    ConnectionStart,
    ConnectionComplete,
    ConnectionAttemptFail,
    InvalidPassword,
    Disconnected,
    Unknown
}

impl Default for NetworkSecurity {
    fn default() -> NetworkSecurity {
        NetworkSecurity::Open
    }
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

pub fn convert_msm_notification_reason(reason_code: u32) -> ConnectionNotifcation {
    match reason_code {
        11 => ConnectionNotifcation::InvalidPassword,
        _ => ConnectionNotifcation::Unknown
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WlanInterfaceState {
    Connected,
    Disconnected,
    Disconnecting,
    Associating,
    Authenticating,
    Discovering,
    AdHocNetworkFormed,
    NotReady,
    Unavailable,
}

impl Default for WlanInterfaceState {
    fn default() -> Self {
        WlanInterfaceState::Unavailable
    }
}

#[allow(non_upper_case_globals)]
pub fn convert_wlan_interface_state(state: WLAN_INTERFACE_STATE) -> WlanInterfaceState {
    let state_enum = match state {
        wlan_interface_state_connected => WlanInterfaceState::Connected,
        wlan_interface_state_disconnected => WlanInterfaceState::Disconnected,
        wlan_interface_state_associating => WlanInterfaceState::Associating,
        wlan_interface_state_authenticating => WlanInterfaceState::Authenticating,
        wlan_interface_state_discovering => WlanInterfaceState::Discovering,
        wlan_interface_state_disconnecting => WlanInterfaceState::Disconnecting,
        wlan_interface_state_ad_hoc_network_formed => WlanInterfaceState::AdHocNetworkFormed,
        wlan_interface_state_not_ready => WlanInterfaceState::NotReady,
        _ => WlanInterfaceState::Unavailable,
    };

    state_enum
}

pub fn check_wlan_interface_state<F, T>(
    state: &WlanInterfaceState,
    mut closure: F) -> Option<T>
where
    F: FnMut() -> T
{
    let c = closure();

   match state {
        WlanInterfaceState::Connected => Some(c),
        WlanInterfaceState::Disconnected => Some(c),
        WlanInterfaceState::Associating => Some(c),
        WlanInterfaceState::Authenticating => Some(c),
        WlanInterfaceState::Discovering => Some(c),
        WlanInterfaceState::Unavailable => None,
        _ => None
    }
}

#[allow(dead_code)]
pub enum NotificationState {
    HasData,
    Disconnected,
    Empty,
    StateKnown
}

impl Default for NotificationState {
    fn default() -> Self {
        NotificationState::Empty
    }
}