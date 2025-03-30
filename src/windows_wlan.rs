use crate::wlan_enums::*;
use std::collections::HashMap;
use std::ffi::c_void;
use windows::Win32::NetworkManagement::WiFi::*;
use windows::Win32::Foundation::{ERROR_SUCCESS, HANDLE, WIN32_ERROR};
use std::ptr::{addr_of, null_mut, NonNull};
use std::rc::Rc;
use std::slice;
use godot::prelude::*;

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

    pub fn open_handle(&mut self) {
        if self.is_handle_open {
            godot_print!("[WLAN] Open Handle Already From Client");
            return;
        }

        unsafe {
            let handle_status = WlanOpenHandle(
                self.client_version,
                None,
                &mut self.negotiated_client_version,
                &mut self.client_handle);

            match check_win32(handle_status) {
                Ok(_) => godot_print!("[WLAN] Open Handle Ok"),
                Err(e) => godot_error!("[WLAN] Open Handle Failed To Open Handle: {:?}", e)
            }
        }

        self.is_handle_open = true;
        godot_print!("[WLAN] Client Handle Opened");
    }

    pub fn wlan_query_interface(&self) -> Option<String>  {
        let mut data_size: u32 = 0;
        let mut data_ptr: *mut c_void = null_mut();
        let mut conn_attribs: Option<&WLAN_CONNECTION_ATTRIBUTES> = None;

        let op_code = wlan_intf_opcode_current_connection;
        let mut op_type = wlan_opcode_value_type_query_only;

        let ifo = self.interface_info.as_ref().unwrap();
        unsafe {
            let query_result = WlanQueryInterface(
                self.client_handle,
                &ifo.InterfaceGuid,
                op_code,
                None,
                &mut data_size,
                &mut data_ptr,
                Some(&mut op_type),
            );

            let result = check_win32(query_result);
            match result {
                Ok(_) => godot_print!("[WLAN] Query Interface Ok"),
                Err(e) => {
                    godot_error!("[WLAN] Query Interface Failed To Query Interface: {:?}", e);
                    return None
                }
            }

            conn_attribs = Some(&*(data_ptr as *const WLAN_CONNECTION_ATTRIBUTES));
        }

        let conn_unwrapped = conn_attribs.unwrap();
        let ssid = conn_unwrapped.wlanAssociationAttributes.dot11Ssid;
        let ssid_raw = &ssid.ucSSID[..ssid.uSSIDLength as usize];
        let ssid_string = String::from_utf8_lossy(ssid_raw).to_string();

        Some(ssid_string)
    }

    pub fn fetch_network_data(&mut self) {
        match self.scan_networks() {
            Ok(()) => {},
            Err(e) => {
                godot_error!("[WLAN] Failed to Get Networks: {:?}", e);
                return;
            }
        };
    }

    pub fn request_scan(&mut self) {
        godot_print!("[WLAN] Requesting Scan");
        self.open_handle();

        match self.interface_info {
            None => {
                godot_warn!("[WLAN] No Interface Info. Retrieving.");
                let interfaces = self.get_interfaces();
                self.retrieve_interface_from_vec(interfaces);
                godot_print!("[WLAN] Done")
            }
            Some(_) => godot_print!("[WLAN] Interface info present. Proceeding."),
        }

        let result = unsafe {
            WlanScan(
                self.client_handle,
                &self.interface_info.unwrap().InterfaceGuid,
                None,
                None,
                None
            )
        };

        match check_win32(result) {
            Ok(_) => godot_print!("[WLAN] Request Scan Ok"),
            Err(e) => godot_error!("[WLAN] Request Scan Failed: {:?}", e)
        }
    }

    pub fn refresh_networks(&mut self) {
        self.networks.clear();
        let ifo = self.interface_info.unwrap();

        let new_network_list = match self.get_available_networks(&ifo) {
            Ok(new_network_list) => new_network_list,
            Err(e) => {
                godot_error!("[WLAN] Failed to Get Available Networks: {:?}", e);
                return;
            }
        };

        for network in new_network_list {
            let ssid_length = network.dot11Ssid.uSSIDLength as usize;
            if ssid_length > 32 {
                continue;
            }

            let net = self.construct_network_object(&network, ssid_length);
            self.networks.insert(net.get_ssid(), net);
        }
    }

    pub fn scan_networks(&mut self) -> Result<(), WIN32_ERROR> {
        if !self.is_handle_open {
            godot_error!("[WLAN] A Client Handle Must Be Open to Scan for Networks!");
            return Ok(());
        }

        let interfaces = self.get_interfaces();

        for int_info in interfaces {
            let networks = self.get_available_networks(&int_info)?;
            let ifo = int_info.clone();

            let check = self.retrieve_interface(ifo);
            if !check.unwrap() {
                continue;
            }

            for net in networks {
                let ssid_length = net.dot11Ssid.uSSIDLength as usize;
                if ssid_length > 32 {
                    continue;
                }

                let network = self.construct_network_object(&net, ssid_length);
                self.networks.insert(network.get_ssid(), network);
            }
        }

        Ok(())
    }

    fn construct_network_object(&self, net: &WLAN_AVAILABLE_NETWORK, ssid_length: usize) -> Network {
        let ssid_bytes = &net.dot11Ssid.ucSSID[..ssid_length];
        let ssid = String::from_utf8_lossy(ssid_bytes);

        let (is_secured, security) = check_security(&net);
        let signal_strength = check_signal_strength(&net);

        let network = Network::new(ssid.to_string(), is_secured, security, signal_strength);
        network
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


// Retrieval of data
impl NetworkManager {
    fn get_interfaces(&self) -> Vec<WLAN_INTERFACE_INFO> {
        unsafe {
            let mut interface_list_ptr: *mut WLAN_INTERFACE_INFO_LIST = null_mut();
            let enum_result = WlanEnumInterfaces(self.client_handle, None, &mut interface_list_ptr);
            match check_win32(enum_result) {
                Ok(_) => godot_print!("[WLAN] Interface List Retrieval Successful"),
                Err(e) => godot_error!("[WLAN] Failed to Retrieve interfaces List: {:?}", e)
            }

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

            interfaces.to_vec()
        }
    }

    fn retrieve_interface_from_vec(&mut self, interfaces: Vec<WLAN_INTERFACE_INFO>) {
        for interface in interfaces {
            let state = convert_wlan_interface_state(interface.isState);
            let ifo = interface.clone();

            if let None = self.interface_info {
                if !check_wlan_interface_state(&state, || {
                    self.interface_info = Some(ifo);
                    true
                }) {
                    continue;
                }
            }
        }
    }

    fn retrieve_interface(&mut self, interface: WLAN_INTERFACE_INFO) -> Option<bool> {
        let state = convert_wlan_interface_state(interface.isState);
        let ifo = interface.clone();

        if let None = self.interface_info {
            let check = check_wlan_interface_state(&state, || {
               self.interface_info = Some(ifo);
                true
            });

            return Some(check);
        }

        None
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