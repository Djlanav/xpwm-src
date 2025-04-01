use crate::globals;
use crate::windows_wlan::NetworkManager;
use crate::wlan_enums::{ConnectionNotifcation, NetworkSecurity};
use godot::prelude::*;
use std::ptr::null_mut;
use std::sync::TryLockError::{Poisoned, WouldBlock};
use std::sync::mpsc::TryRecvError::{Disconnected, Empty};
use windows::Win32::Foundation::HANDLE;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct WlanAPI {
    network_manager: NetworkManager,
    base: Base<Node>
}

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct WiFiNetwork {
    #[var]
    ssid: GString,
    #[var]
    secured: bool,
    #[var]
    network_security: NetworkSecurity,
    #[var]
    bars: u32
}

#[godot_api]
impl INode for WlanAPI {
    fn init(base: Base<Node>) -> Self {
        godot_print!("[WLAN] Initializing WlanAPI");
        godot_print!("[WLAN] Initializing NetworkManager");

        Self {
            network_manager: NetworkManager::new(HANDLE(null_mut()), 2),
            base
        }
    }

    fn exit_tree(&mut self) {
        self.close_wlan_handle();
    }
}

#[godot_api]
impl WlanAPI {
    #[signal]
    fn network_data_fetched();

    #[signal]
    fn connection_status_received(status: ConnectionNotifcation);

    #[signal]
    fn funny_signal();

    #[func]
    fn fetch_network_data(&mut self) {
        self.network_manager.open_handle();
        godot_print!("[WLAN] NetworkManager Ready");
        godot_print!("[WLAN] Scanning for Networks");

        self.network_manager.fetch_network_data();
        self.signals().network_data_fetched().emit();
    }

    #[func]
    fn poll_connection_status(&self) -> Variant {
        let status_guard = match globals::CONNECTION_NOTIFICATION_CHANNEL.try_lock() {
            Ok(g) => g,
            Err(error) => match error {
                Poisoned(poison_error) => poison_error.into_inner(),
                WouldBlock => {
                    godot_print!("[WLAN] Status Is Currently Locked. Continuing.");
                    return Variant::nil();
                },
            },
        };

        let status = match status_guard.1.try_recv() {
            Ok(status_enum) => status_enum,
            Err(error) => {
                match error {
                    Empty => {
                        godot_warn!("[WLAN] Notification Receiver Was Empty");
                        return Variant::nil();
                    },
                    Disconnected => {
                        godot_error!("[WLAN] Notification Receiver Was Disconnected");
                        return Variant::nil();
                    },
                }
            },
        };

        Variant::from(status.to_godot())
    }

    #[func]
    fn connect(&self, ssid: GString) {
        if ssid == GString::from("Linksys775") {
            let ssid_string = ssid.to_string();
            self.network_manager.connect_to_known_network(ssid_string.as_str());
        } else {
            godot_error!("[WLAN] Can Only Connect To Known Networks At This Time");
        }
    }

    #[func]
    fn disconnect(&self) {
        self.network_manager.disconnect_from_network();
    }

    #[func]
    fn scan_networks(&mut self) {
        self.network_manager.request_scan();
    }

    #[func]
    fn refresh_network_data(&mut self) {
        godot_print!("[WLAN] Refreshing NetworkData");

        self.network_manager.refresh_networks();
        self.signals().network_data_fetched().emit();
    }

    #[func]
    fn check_connectivity(&self) -> GString {
        let ssid = match self.network_manager.wlan_query_interface() {
            Some(ssid) => ssid,
            None => return GString::new(),
        };

        GString::from(ssid)
    }

    #[func]
    fn get_networks(&self) -> Dictionary {
        let networks = self.network_manager.get_networks();
        let mut networks_dictionary = Dictionary::new();

        for (ssid, network) in networks {
            let mut wifi_network = WiFiNetwork::new_gd();
            let mut wifi_bind = wifi_network.bind_mut();

            wifi_bind.ssid = GString::from(ssid.as_ref());
            wifi_bind.network_security = network.get_security();
            wifi_bind.secured = network.get_secured();
            wifi_bind.bars = network.get_bars();

            let wifi_ssid = wifi_bind.ssid.clone();
            drop(wifi_bind);

            networks_dictionary.set(wifi_ssid, wifi_network);
        }

        networks_dictionary
    }

    #[func]
    fn close_wlan_handle(&self) {
        match self.network_manager.close_handle() {
            Ok(_) => godot_print!("[WLAN] Closing WlanHandle"),
            Err(e) => godot_print!("[WLAN] Failed to Close WlanHandle: {:?}", e)
        }
    }
}