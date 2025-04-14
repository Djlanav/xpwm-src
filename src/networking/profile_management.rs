use std::{mem::ManuallyDrop, ptr::addr_of};
use std::slice;

use windows::Win32::NetworkManagement::WiFi::*;
use godot::prelude::*;

use crate::windows_api::{convert_string_to_u16cstring, convert_u16_slice_to_u16cstring, wlan};

use super::NetworkManager;

impl NetworkManager {
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

    pub fn set_wlan_profile(&self, profile: &String) {
        let ifo = self.interface_info.unwrap().InterfaceGuid;
        let profile_u16 = convert_string_to_u16cstring(profile).unwrap();

        let set_profile_result = wlan::set_profile(self.client_handle, &ifo, &profile_u16, true);
        if let Err(error) = set_profile_result {
            godot_error!("[DEBUG] Failed To Set Profile: {:?}", error);
            return;
        };
    }

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
}