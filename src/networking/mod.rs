pub mod connecting;
pub mod scanning;
pub mod profile_management;
pub mod interface_management;
pub mod adapter_checking;

use std::{collections::HashMap, rc::Rc};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::{Foundation::HANDLE, NetworkManagement::WiFi::*};
use godot::prelude::*;

use crate::callbacks;
use crate::windows_api::wlan;
use crate::wlan_enums::*;
use crate::utils::*;

pub struct NetworkManager {
    pub networks: HashMap<Rc<String>, Network>,
    pub interface_info: Option<WLAN_INTERFACE_INFO>,
    pub client_handle: HANDLE,
    pub is_handle_open: bool,
    pub client_version: u32,
    pub negotiated_client_version: u32,
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

    fn register_wlan_notification(&self) {
        let handle = self.client_handle;
        let sources = WLAN_NOTIFICATION_SOURCE_ACM | WLAN_NOTIFICATION_SOURCE_MSM;

        wlan::register_notification(handle, sources, false, Some(callbacks::wlan_acm_notification_callback));
    }

    pub fn get_network(&self, ssid: &str) -> &Network {
        let network = self.networks.get(&ssid.to_string()).unwrap();
        network
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

#[derive(Clone)]
#[allow(dead_code)]
pub struct Network {
    pub ssid: Rc<String>,
    pub secured: bool,
    pub connected: bool,
    pub network_security: NetworkSecurity,
    pub encryption: EncryptionAlgorithm,
    pub bars: u32
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