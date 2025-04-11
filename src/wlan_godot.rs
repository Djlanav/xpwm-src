use crate::globals;
use crate::profile_management::generate_network_profile_xml;
use crate::windows_api::{convert_string_to_u16cstring, convert_u16_slice_to_string, wlan};
use crate::windows_wlan::NetworkManager;
use crate::wlan_enums::{ConnectionNotifcation, NetworkSecurity, NotificationState, WlanInterfaceState};
use godot::prelude::*;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::Path;
use std::ptr::null_mut;
use std::sync::TryLockError::{Poisoned, WouldBlock};
use std::sync::mpsc::TryRecvError::{Disconnected, Empty};
use windows::Win32::Foundation::HANDLE;

#[derive(GodotClass)]
#[class(base=Object)]
pub struct WlanAPI {
    network_manager: NetworkManager,
    notif_state: NotificationState,
    interface_state: WlanInterfaceState,
    known_networks: Array<GString>,
    base: Base<Object>
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
    bars: u32,
}

#[godot_api]
impl IObject for WlanAPI {
    fn init(base: Base<Object>) -> Self {
        godot_print!("[WLAN] Initializing NetworkManager");

        Self {
            network_manager: NetworkManager::new(HANDLE(null_mut()), 2),
            notif_state: NotificationState::default(),
            interface_state: WlanInterfaceState::default(),
            known_networks: Array::new(),
            base
        }
    }
}

#[godot_api]
impl WlanAPI {
    #[signal]
    fn network_data_fetched();

    #[signal]
    fn windows_profiles_found(ssid: GString);

    // Connection Status Signals
    #[signal]
    fn connection_start();

    #[signal]
    fn connection_complete();

    #[signal]
    fn connection_attempt_fail();

    #[signal]
    fn invalid_password();

    #[signal]
    fn disconnected();

    #[cfg(debug_assertions)]
    #[func]
    fn test_xml_data(ssid: GString) {
        globals::save_xml_to_disk(ssid.to_string().as_str());
    }

    #[func]
    fn initialize_network_manager(&mut self) {
        self.network_manager.init();
    }

    #[func]
    fn fetch_network_data(&mut self) {
        self.network_manager.open_handle();
        godot_print!("[WLAN] NetworkManager Ready");
        godot_print!("[WLAN] Scanning for Networks");

        self.network_manager.fetch_network_data();
        self.signals().network_data_fetched().emit();
    }

    #[func]
    fn poll_connection_status(&mut self) {
        let status_guard = match globals::CONNECTION_NOTIFICATION_CHANNEL.try_lock() {
            Ok(g) => g,
            Err(error) => match error {
                Poisoned(poison_error) => poison_error.into_inner(),
                WouldBlock => {
                    godot_warn!("[WLAN] Status Is Currently Locked. Continuing.");
                    return;
                },
            },
        };

        match status_guard.1.try_recv() {
            Ok(status_enum) => {
                match status_enum {
                    ConnectionNotifcation::ConnectionStart => {
                        godot_print!("Got Data From Notification Receiver: {:?}", status_enum);
                        self.signals().connection_start().emit();
                    }
                    ConnectionNotifcation::ConnectionComplete => { 
                        godot_print!("Got Data From Notification Receiver: {:?}", status_enum);
                        self.signals().connection_complete().emit();
                        self.interface_state = WlanInterfaceState::Connected 
                    },
                    ConnectionNotifcation::Disconnected => {
                        godot_print!("Got Data From Notification Receiver: {:?}", status_enum);
                        self.signals().disconnected().emit();
                        self.interface_state = WlanInterfaceState::Disconnected
                    },
                    ConnectionNotifcation::Unknown => self.interface_state = WlanInterfaceState::Unavailable,
                    _ => {}
                }

                status_enum
            },
            Err(error) => {
                match error {
                    Empty => {
                        if let NotificationState::Empty = self.notif_state {
                            godot_warn!("[WLAN] Receiver Was Empty");
                            self.notif_state = NotificationState::StateKnown;
                            return;
                        }

                        return;
                    },
                    Disconnected => {
                        if let NotificationState::Disconnected = self.notif_state {
                            godot_error!("[WLAN] Notification Receiver Was Disconnected");
                            self.notif_state = NotificationState::StateKnown;
                            return;
                        }

                        return;
                    },
                }
            },
        };
    }

    #[func]
    fn add_network_to_known_networks(&mut self, ssid: GString) {
        let file_path = Path::new("wlan_data").join("known_networks.txt");
        if file_path.exists() {
            godot_warn!("[SYSTEM] File 'known_networks.txt' Already Exists. Returning.");
            return;
        }

        if self.known_networks.contains(&ssid.to_string()) {
            return;
        }

        if let Some(parent) = file_path.parent() {
            match create_dir_all(parent) {
                Ok(_) => godot_print!("[SYSTEM] Successfully Created WLAN Data Directory."),
                Err(error) => {
                    godot_error!("[SYSTEM] Failed To Create Directory For WLAN Data. Error: {}", error);
                    return;
                },
            }
        }

        let mut networks_file = match File::create(file_path) {
            Ok(file) => file,
            Err(error) => {
                godot_error!("[SYSTEM] Failed To Create File 'known_networks.txt'. Error: {}", error);
                return;
            },
        };

        let ssid_string = ssid.to_string();
        match networks_file.write_all(format!("{}\n", ssid_string).as_bytes()) {
            Ok(_) => godot_print!("[SYSTEM] Wrote Data To File 'known_networks.txt'."),
            Err(error) => godot_error!("[SYSTEM] Failed To Write Data To 'known_networks.txt'. Error: {}", error),
        };

        self.known_networks.push(&GString::from(ssid_string));
    }

    #[func]
    pub fn read_from_known_networks(&mut self) {
        let mut networks_file = match File::open("wlan_data/known_networks.txt") {
            Ok(file) => file,
            Err(error) => {
                godot_error!("[SYSTEM] Failed To Open File 'known_networks.txt'. Error: {}", error);
                return;
            },
        };
        let mut known_networks = String::new();
        match networks_file.read_to_string(&mut known_networks) {
            Ok(_) => godot_print!("[SYSTEM] Successfully Read From File 'known_networks.txt'."),
            Err(error) => {
                godot_error!("[SYSTEM] Failed To Read From File 'known_networks.txt'. Error: {}", error);
                return;
            },
        };

        let s: Vec<GString> = known_networks
        .lines()
        .map(|network| network.trim().to_string())
        .filter(|network| !network.is_empty())
        .map(|s| GString::from(s))
        .collect();

        self.known_networks = Array::from(s.as_slice());
    }

    #[func]
    fn connect(&self, ssid: GString) {
        let ssid_string = ssid.to_string();
        self.network_manager.connect_to_known_network(&ssid_string);
    }

    #[func]
    fn generate_profile(&self, ssid: GString, password: GString) {
        let (ssid_string, password_string) = (ssid.to_string(), password.to_string());
        let network = self.network_manager.get_network(ssid.to_string().as_str());

        let profie = generate_network_profile_xml(ssid_string.as_str(), password_string.as_str(), &network.get_encryption(), &network.get_security());
        self.network_manager.set_wlan_profile(&profie);
    }

    #[func]
    fn check_for_matching_profile(&self, ssid: GString) -> bool {
        let ifo = self.network_manager.get_interface_info().unwrap();

        let profiles = match self.network_manager.get_profile_list(ifo) {
            Some(list) => list,
            None => {
                godot_error!("[WLAN] Failed To Retrieve Profile List");
                return false;
            },
        };

        let match_found = profiles.iter().any(|p| {
            let rust_string = convert_u16_slice_to_string(&p.strProfileName);
            let string = GString::from(rust_string);

            ssid == string
        });

        match_found
    }

    #[func]
    fn check_for_windows_profiles(&mut self) {
        let ifo = self.network_manager.get_interface_info().unwrap();

        if let Some(profile_list) = self.network_manager.get_profile_list(ifo) {
            if let Some((ssid, _)) = self.network_manager.check_for_windows_profiles(&profile_list) {
                let ssid_gstring = GString::from(ssid);
                self.signals().windows_profiles_found().emit(ssid_gstring);
            }
        }
    }

    #[func]
    fn delete_profile(&self, ssid: String) {
        let handle = self.network_manager.get_client_handle();
        let ifo = self.network_manager.get_interface_info().unwrap();

        let wide_ssid = match convert_string_to_u16cstring(&ssid) {
            Some(wide) => wide,
            None => return
        };

        wlan::delete_profile(handle, &ifo.InterfaceGuid, &wide_ssid);
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
    fn check_for_active_connection(&mut self) -> bool {
        match self.network_manager.check_for_active_connection() {
            Some(_) => {
                godot_print!("[WLAN] Interface Is Connected");
                self.interface_state = WlanInterfaceState::Connected;
                true
            },
            None => {
                godot_print!("[WLAN] Interface Is Disconnected");
                self.interface_state = WlanInterfaceState::Disconnected;
                false
            },
        }
    }

    #[func]
    fn get_connected_ssid(&self) -> Variant {
        let result = match self.interface_state {
            WlanInterfaceState::Connected => {
                let ssid = match self.network_manager.get_connected_network() {
                    Some(ssid) => Variant::from(ssid),
                    None => Variant::nil(),
                };

                ssid
            }

            WlanInterfaceState::Disconnected => {
                Variant::nil()
            }

            _ => {
                godot_error!("ERROR: REACHED WILD CARD OF INTERFACE STATE MATCH IN GET CONNECTED SSID {:?}", self.interface_state);
                Variant::nil()
            }
        };

        result
    }

    #[allow(unused)]
    fn check_network_connectivity(&self, _ssid: GString) {
        todo!();
    }

    #[func]
    fn is_known_network(&self, ssid: GString) -> bool {
        if self.known_networks.contains(&ssid.to_string()) {
            true
        } else {
            false
        }
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