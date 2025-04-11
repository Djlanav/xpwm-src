use crate::windows_api::{convert_string_to_u16cstring, convert_u16_slice_to_u16cstring, wlan};
use crate::{callbacks, wlan_enums::*};
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::ManuallyDrop;
use widestring::U16CString;
use windows::core::PCWSTR;
use windows::Win32::NetworkManagement::WiFi::*;
use windows::Win32::Foundation::{ERROR_SUCCESS, HANDLE, WIN32_ERROR};
use std::ptr::{addr_of, null_mut, NonNull};
use std::rc::Rc;
use std::slice;
use godot::prelude::*;

#[derive(Clone)]
#[allow(dead_code)]
pub struct Network {
    ssid: Rc<String>,
    secured: bool,
    connected: bool,
    network_security: NetworkSecurity,
    encryption: EncryptionAlgorithm,
    bars: u32
}

impl Network {
    pub fn new(
        ssid: String, 
        secured: bool, 
        connected: bool, 
        network_security: NetworkSecurity,
        encryption: EncryptionAlgorithm,
        bars: u32,) -> Self 
    {
        Network {
            ssid: Rc::new(ssid),
            secured,
            connected,
            network_security,
            encryption,
            bars
        }
    }

    pub fn get_ssid(&self) -> Rc<String> {
        self.ssid.clone()
    }

    pub fn get_security(&self) -> NetworkSecurity {
        self.network_security.clone()
    }

    pub fn get_encryption(&self) -> EncryptionAlgorithm {
        self.encryption.clone()
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
            Ok(_) => godot_print!("[WLAN] Client Handle Closed. NetworkManager Done"),
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

    pub fn get_client_handle(&self) -> HANDLE {
        self.client_handle.clone()
    }

    pub fn get_interface_info(&self) -> Option<&WLAN_INTERFACE_INFO> {
        self.interface_info.as_ref()
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

    pub fn init(&mut self) {
        self.open_handle();
        self.initialize_interface_info();
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
                Ok(_) => {
                    godot_print!("[WLAN] Open Handle Ok");
                    self.register_wlan_notification();
                },
                Err(e) => godot_error!("[WLAN] Open Handle Failed To Open Handle: {:?}", e)
            }
        }

        self.is_handle_open = true;
        godot_print!("[WLAN] Client Handle Opened");
    }

    fn register_wlan_notification(&self) {
        let handle = self.client_handle;
        let sources = WLAN_NOTIFICATION_SOURCE_ACM | WLAN_NOTIFICATION_SOURCE_MSM;

        wlan::register_notification(handle, sources, false, Some(callbacks::wlan_acm_notification_callback));
    }

    pub fn check_for_active_connection(&self) -> Option<&WLAN_CONNECTION_ATTRIBUTES> {
        let mut data_size = 0u32;

        let op_code = wlan_intf_opcode_current_connection;
        let mut op_type = wlan_opcode_value_type_query_only;

        let ifo = self.interface_info.as_ref().unwrap();
        let client_handle = self.client_handle;

        let query_result = match wlan::query_interface(client_handle, &ifo.InterfaceGuid, op_code, &mut data_size, &mut op_type) {
            Ok(attribs) => attribs,
            Err(error) => match error {
                wlan::WlanError::Error(err) => {
                    godot_error!("{}", err);
                    return None;
                },
                wlan::WlanError::Win32Error(win32_error) => {
                    godot_error!("[WLAN] Failed To Query Interface: {:?}", win32_error);
                    return None;
                },
            },
        };

        query_result
    }

    pub fn check_for_windows_profiles(&self, profiles: &Vec<WLAN_PROFILE_INFO>) -> Option<(String, bool)> {
        let client_handle = self.client_handle;
        let guid = self.interface_info.unwrap().InterfaceGuid;

        for profile in profiles {
            let u16_cstring = match convert_u16_slice_to_u16cstring(&profile.strProfileName) {
                Some(cstring) => cstring,
                None => return None,
            };

            let retrieved_profile = match wlan::get_profile(client_handle, &guid, &u16_cstring) {
                Ok(ret) => ret.to_string().unwrap(),
                Err(error) => {
                    godot_error!("[WLAN] Failed To Get Profile: {:?}", error);
                    return None;
                },
            };
            
            let check = retrieved_profile.contains("<connectionMode>auto</connectionMode");
            if check {
                return Some((u16_cstring.to_string().unwrap(), check));
            } else {
                return None;
            }
        }

        None
    }

    #[allow(unused_assignments)]
    pub fn get_connected_network(&self) -> Option<String>  {
        if !self.is_handle_open || self.client_handle.is_invalid() {
            godot_error!("[WLAN] Handle Is Either Not Open Or Invalid");
            return None;
        }

        let conn_attribs = match self.check_for_active_connection() {
            Some(attribs) => attribs,
            None => return None,
        };

        let ssid = conn_attribs.wlanAssociationAttributes.dot11Ssid;
        let ssid_raw = &ssid.ucSSID[..ssid.uSSIDLength as usize];
        let ssid_string = String::from_utf8_lossy(ssid_raw).to_string();

        unsafe {
            godot_print!("[WLAN] Freeing query memory");
            WlanFreeMemory(addr_of!(*conn_attribs) as *const c_void);
        }
        Some(ssid_string)
    }

    pub fn connect_to_known_network(&self, ssid: &str) {
        godot_print!("[WLAN] Connecting To Known Network: {}", ssid);

        let profile_name = U16CString::from_str(ssid).unwrap();
        let conn_params = WLAN_CONNECTION_PARAMETERS {
            wlanConnectionMode: wlan_connection_mode_profile,
            strProfile: PCWSTR::from_raw(profile_name.as_ptr()),
            dot11BssType: dot11_BSS_type_infrastructure,
            pDot11Ssid: null_mut(),
            pDesiredBssidList: null_mut(),
            dwFlags: 0
        };
        
        let ifo = self.interface_info.unwrap();
        match wlan::connect(self.client_handle, &ifo.InterfaceGuid, &conn_params) {
            Ok(_) => godot_print!("[WLAN] Connected to Network: {}", ssid),
            Err(e) => godot_error!("[WLAN] Failed to Connect to Network: {:?}", e),
        }
    }

    // TESTING PURPOSES
    pub fn get_network(&self, ssid: &str) -> &Network {
        let network = self.networks.get(&ssid.to_string()).unwrap();
        network
    }

    pub fn set_wlan_profile(&self, profile: &String) {
        let ifo = self.interface_info.unwrap().InterfaceGuid;
        let profile_u16 = convert_string_to_u16cstring(profile).unwrap();

        match wlan::set_profile(self.client_handle, &ifo, &profile_u16, true) {
            Ok(_) => godot_print!("[DEBUG] Profile Was Successfully Set"),
            Err(error) => {
                godot_error!("[DEBUG] Failed To Set Profile: {:?}", error);
                return;
            },
        };
    }

    pub fn disconnect_from_network(&self) {
        godot_print!("[WLAN] Disconnecting From Network");
        let ifo = self.interface_info.unwrap();

        wlan::disconnect(self.client_handle, &ifo.InterfaceGuid);
    }

    pub fn fetch_network_data(&mut self) {
        if let Err(e) = self.scan_networks() {
            godot_error!("[WLAN] Failed To Get Networks: {:?}", e);
            return;
        }
    }

    pub fn initialize_interface_info(&mut self) {
        match self.interface_info {
            None => {
                godot_warn!("[WLAN] No Interface Info. Retrieving.");
                let interfaces = self.get_interfaces();
                self.retrieve_interface_from_vec(interfaces);
                godot_print!("[WLAN] Got Interface Info. Continuing With Scan");
            }
            Some(_) => godot_print!("[WLAN] Interface Info Present. Proceeding."),
        }
    }

    pub fn request_scan(&mut self) {
        godot_print!("[WLAN] Requesting Scan");
        let handle = self.client_handle;
        let ifo = self.interface_info.unwrap().InterfaceGuid;

        wlan::scan(handle, &ifo);
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
        let net_encryption = check_encryption(&net);

        let network = Network::new(
            ssid.to_string(), 
            is_secured, false, 
            security,
            net_encryption,
            signal_strength);
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
    pub fn get_profile_list(&self, ifo: &WLAN_INTERFACE_INFO) -> Option<Vec<WLAN_PROFILE_INFO>> {
        let profile_list = match wlan::get_profile_list(self.client_handle, &ifo.InterfaceGuid) {
            Ok(result) => result,
            Err(error) => {
                godot_error!("[WLAN] Failed To Get Profile List: {:?}", error);
                return None;
            },
        };

        let list_length = profile_list.dwNumberOfItems;
        let info_ptr = addr_of!(profile_list.ProfileInfo);
        let info_slice = unsafe {
            slice::from_raw_parts(info_ptr.cast::<WLAN_PROFILE_INFO>(), list_length as usize)
        }.to_vec();

        unsafe {
            let list_box = ManuallyDrop::into_inner(profile_list);
            let raw_ptr = Box::into_raw(list_box);
            WlanFreeMemory(raw_ptr as _);
        }
        Some(info_slice)
    }

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

            let interface_vec = interfaces.to_vec();

            godot_print!("[WLAN] Freeing interfaces list memory");
            WlanFreeMemory(interface_list_ptr.cast());

            interface_vec
        }
    }

    fn retrieve_interface_from_vec(&mut self, interfaces: Vec<WLAN_INTERFACE_INFO>) {
        for interface in interfaces {
            let state = convert_wlan_interface_state(interface.isState);
            let ifo = interface.clone();

            if let None = self.interface_info {
                if let None =  check_wlan_interface_state(&state, || {
                    self.interface_info = Some(ifo);
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
            match check_wlan_interface_state(&state, || {
               self.interface_info = Some(ifo);
               true
            }) {
                Some(result) => return Some(result),
                None => return None,
            }
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

            let networks_vec = networks.to_vec();
            godot_print!("[WLAN] Freeing networks list memory");
            WlanFreeMemory(network_list_ptr.cast());

            Ok(networks_vec)
        }
    }
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