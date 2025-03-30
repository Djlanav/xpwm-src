use crate::wlan_enums::*;
use std::collections::HashMap;
use windows::Win32::NetworkManagement::WiFi::*;
use windows::Win32::Foundation::{ERROR_SUCCESS, HANDLE, WIN32_ERROR};
use std::ptr::{addr_of, null_mut, NonNull};
use std::rc::Rc;
use std::slice;
use godot::prelude::*;

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

#[derive(Clone)]
pub struct Network {
    ssid: Rc<String>,
    secured: bool,
    network_security: NetworkSecurity,
    bars: u32
}

impl Network {
    pub fn new(ssid: String, secured: bool, network_security: NetworkSecurity, bars: u32) -> Self {
        Network {
            ssid: Rc::new(ssid),
            secured,
            network_security,
            bars
        }
    }

    pub fn get_ssid(&self) -> Rc<String> {
        self.ssid.clone()
    }

    pub fn get_security(&self) -> NetworkSecurity {
        self.network_security.clone()
    }

    pub fn get_bars(&self) -> u32 {
        self.bars
    }

    pub fn get_secured(&self) -> bool {
        self.secured
    }
}

pub struct NetworkManager {
    networks: HashMap<Rc<String>, Network>,
    interface_info: Option<WLAN_INTERFACE_INFO>,
    client_handle: HANDLE,
    is_handle_open: bool,
    client_version: u32,
    negotiated_client_version: u32,
}

impl Drop for NetworkManager {
    fn drop(&mut self) {
        match self.close_handle() {
            Ok(_) => godot_print!("[WLAN] NetworkManager Done"),
            Err(e) => {
                godot_error!("[WLAN] NetworkManager Failed To Close Client Handle: {:?}", e);
            }
        }
    }
}

impl NetworkManager {
    pub fn get_networks(&self) -> HashMap<Rc<String>, Network> {
        self.networks.clone()
    }
}

impl NetworkManager {
    pub fn new(client_handle: HANDLE, client_version: u32) -> Self {
        Self {
            networks: HashMap::new(),
            interface_info: None,
            client_handle,
            is_handle_open: false,
            client_version,
            negotiated_client_version: 0,
        }
    }

    pub fn open_handle(&mut self) -> Result<(), WIN32_ERROR> {
        if self.is_handle_open {
            godot_print!("[WLAN] Open Handle Already From Client");
            return Ok(())
        }

        unsafe {
            let handle_status = WlanOpenHandle(
                self.client_version,
                None,
                &mut self.negotiated_client_version,
                &mut self.client_handle);

            check_win32(handle_status)?;
        }

        self.is_handle_open = true;
        godot_print!("[WLAN] Client Handle Opened");
        Ok(())
    }


    pub fn fetch_network_data(&mut self) {
        let networks = match self.scan_networks() {
            Ok(networks) => networks.unwrap(),
            Err(e) => {
                godot_error!("[WLAN] Failed to Get Networks: {:?}", e);
                return;
            }
        };

        self.networks = networks;
    }

    pub fn request_scan(&self) -> Result<(), WIN32_ERROR> {
        godot_print!("[WLAN] Requesting Scan");

        let result = unsafe {
            WlanScan(
                self.client_handle,
                &self.interface_info.unwrap().InterfaceGuid,
                None,
                None,
                None
            )
        };

        check_win32(result)?;
        Ok(())
    }

    pub fn scan_networks(&mut self) -> Result<Option<HashMap<Rc<String>, Network>>, WIN32_ERROR> {
        if !self.is_handle_open {
            godot_error!("[WLAN] A Client Handle Must Be Open to Scan for Networks!");
            return Ok(None);
        }

        let mut networks_hashmap = HashMap::new();
        let interfaces = self.get_interfaces()?;

        for int_info in interfaces {
            let networks = self.get_available_networks(&int_info)?;
            let state = convert_wlan_interface_state(int_info.isState);
            let ifo = int_info.clone();

            if let None = self.interface_info {
                match state {
                    WlanInterfaceState::Connected => self.interface_info = Some(ifo),
                    WlanInterfaceState::Disconnected => self.interface_info = Some(ifo),
                    WlanInterfaceState::Associating => self.interface_info = Some(ifo),
                    WlanInterfaceState::Authenticating => self.interface_info = Some(ifo),
                    WlanInterfaceState::Discovering => self.interface_info = Some(ifo),
                    WlanInterfaceState::Unavailable => continue
                }
            }

            for net in networks {
                let ssid_length = net.dot11Ssid.uSSIDLength as usize;
                if ssid_length > 32 {
                    continue;
                }

                let ssid_bytes = &net.dot11Ssid.ucSSID[..ssid_length];
                let ssid = String::from_utf8_lossy(ssid_bytes);

                let (is_secured, security) = check_security(&net);
                let signal_strength = check_signal_strength(&net);

                let network = Network::new(ssid.to_string(), is_secured, security, signal_strength);
                networks_hashmap.insert(network.get_ssid(), network);
            }
        }

        Ok(Some(networks_hashmap))
    }

    fn get_interfaces(&self) -> Result<Vec<WLAN_INTERFACE_INFO>, WIN32_ERROR> {
        unsafe {
            let mut interface_list_ptr: *mut WLAN_INTERFACE_INFO_LIST = null_mut();
            let enum_result = WlanEnumInterfaces(self.client_handle, None, &mut interface_list_ptr);
            check_win32(enum_result)?;

            let interface_list = match NonNull::new(interface_list_ptr) {
                Some(interface_list) => interface_list,
                None => panic!("[WLAN] Interface List pointer was null"),
            };

            let interface_ref = interface_list.as_ref();
            let interfaces_len = interface_ref.dwNumberOfItems as usize;
            let interfaces_ptr = addr_of!(interface_ref.InterfaceInfo);

            let interfaces = slice::from_raw_parts(
                interfaces_ptr.cast::<WLAN_INTERFACE_INFO>(),
                interfaces_len);

            Ok(interfaces.to_vec())
        }
    }

    fn get_available_networks(
        &self,
        interface_info: &WLAN_INTERFACE_INFO
        ) -> Result<Vec<WLAN_AVAILABLE_NETWORK> , WIN32_ERROR>
    {
        unsafe {
            let mut network_list_ptr: *mut WLAN_AVAILABLE_NETWORK_LIST = null_mut();
            let result = WlanGetAvailableNetworkList(
                self.client_handle,
                &interface_info.InterfaceGuid,
                0,
                None,
                &mut network_list_ptr
            );
            check_win32(result)?;

            let network_list = match NonNull::new(network_list_ptr) {
                Some(ptr) => ptr,
                None => panic!("LIST PTR IS NULL")
            };
            let networks_ref = network_list.as_ref();

            let networks_len = networks_ref.dwNumberOfItems;
            let networks_ptr = addr_of!(networks_ref.Network);

            let networks = slice::from_raw_parts(
                networks_ptr.cast::<WLAN_AVAILABLE_NETWORK>(),
                networks_len as usize
            );

            Ok(networks.to_vec())
        }
    }

    pub fn close_handle(&self) -> Result<(), WIN32_ERROR> {
        if self.is_handle_open {
            unsafe {
                let status = WlanCloseHandle(self.client_handle, None);
                check_win32(status)?;
            }
        } else {
            godot_warn!("[WLAN] Attempted to Close a Non-Open Handle");
        }

        Ok(())
    }
}

fn check_security(network: &WLAN_AVAILABLE_NETWORK) -> (bool, NetworkSecurity) {
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

fn check_signal_strength(network: &WLAN_AVAILABLE_NETWORK) -> u32 {
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