use std::{ptr::{addr_of, null_mut, NonNull}, slice};

use windows::Win32::{Foundation::WIN32_ERROR, NetworkManagement::WiFi::*};
use godot::prelude::*;

use crate::{utils::*, windows_api::wlan, wlan_enums::*};

use super::{Network, NetworkManager};

impl NetworkManager {
    pub fn request_scan(&mut self) {
        godot_print!("[WLAN] Requesting Scan");
        let handle = self.client_handle;
        let ifo = self.interface_info.unwrap().InterfaceGuid;

        wlan::scan(handle, &ifo);
    }

    pub fn refresh_networks(&mut self) {
        if !self.is_handle_open {
           godot_error!("[WLAN] A Client Handle Must Be Open to Scan for Networks!");
           return;
        }

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

    pub fn construct_network_object(&self, net: &WLAN_AVAILABLE_NETWORK, ssid_length: usize) -> Network {
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