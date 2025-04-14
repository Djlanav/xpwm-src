use std::{ffi::c_void, ptr::{addr_of, null_mut}};

use windows::{core::PCWSTR, Win32::NetworkManagement::WiFi::*};
use widestring::*;
use godot::prelude::*;

use crate::windows_api::wlan::{self, WlanError};

use super::NetworkManager;

impl NetworkManager {
    pub fn check_for_active_connection(&self) -> Option<&WLAN_CONNECTION_ATTRIBUTES> {
        let mut data_size = 0u32;

        let op_code = wlan_intf_opcode_current_connection;
        let mut op_type = wlan_opcode_value_type_query_only;

        let ifo = self.interface_info.as_ref().unwrap();
        let client_handle = self.client_handle;

        let query_result = match wlan::query_interface(client_handle, &ifo.InterfaceGuid, op_code, &mut data_size, &mut op_type) {
            Ok(attribs) => attribs,
            Err(error) => match error {
                WlanError::Error(err) => {
                    godot_error!("{}", err);
                    return None;
                },
                WlanError::Win32Error(win32_error) => {
                    godot_error!("[WLAN] Failed To Query Interface: {:?}", win32_error);
                    return None;
                },
            },
        };

        query_result
    }

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
        let connect_result = wlan::connect(self.client_handle, &ifo.InterfaceGuid, &conn_params);

        if let Err(error) = connect_result {
            godot_error!("[WLAN] Failed to Connect to Network: {:?}", error);
            return;
        } else {
            godot_print!("[WLAN] Connected to Network: {}", ssid);
        }
    }

    pub fn disconnect_from_network(&self) {
        godot_print!("[WLAN] Disconnecting From Network");
        let ifo = self.interface_info.unwrap().InterfaceGuid;

        wlan::disconnect(self.client_handle, &ifo);
    }
}