use std::{mem::ManuallyDrop, ptr::addr_of, slice};

use windows::Win32::NetworkManagement::WiFi::*;
use godot::prelude::*;
use crate::{windows_api::wlan::{self, WlanResult}, wlan_enums::*};

use super::NetworkManager;

impl NetworkManager {
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

    fn get_interfaces(&self) -> Option<Vec<WLAN_INTERFACE_INFO>> {
        let client_handle = self.client_handle;
        let interface_list = match wlan::enumerate_interfaces(client_handle) {
            WlanResult::Value(v) => v,
            WlanResult::Error(wlan_error) => {
                wlan_error.check("[WLAN] Failed To Enumerate Interfaces]");
                return None;
            },
        };

        let interfaces_len = interface_list.dwNumberOfItems as usize;
        let interfaces_ptr = addr_of!(interface_list.InterfaceInfo);
        unsafe {
            let interfaces = slice::from_raw_parts(
                interfaces_ptr.cast::<WLAN_INTERFACE_INFO>(),
                interfaces_len);

            let interface_vec = interfaces.to_vec();

            let list_box = ManuallyDrop::into_inner(interface_list);
            let raw_ptr = Box::into_raw(list_box);
            WlanFreeMemory(raw_ptr as _);

            Some(interface_vec)
        }
    }

    pub fn initialize_interface_info(&mut self) {
        if self.interface_info.is_none() {
            godot_warn!("[WLAN] No Interface Info. Retrieving.");
            let interfaces = match self.get_interfaces() {
                Some(ifo) => ifo,
                None => return,
            };


            self.retrieve_interface_from_vec(interfaces);
            godot_print!("[WLAN] Got Interface Info. Continuing With Scan");
        } else {
            godot_print!("[WLAN] Interface Info Present. Proceeding.");
        }
    }
}