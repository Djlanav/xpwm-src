use godot::prelude::*;
use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::ptr::null_mut;

use widestring::{U16CString, U16String};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::NetworkManagement::WiFi::{WlanConnect, WlanDeleteProfile, WlanDisconnect, WlanGetProfile, WlanGetProfileList, WlanQueryInterface, WlanReasonCodeToString, WlanRegisterNotification, WlanScan, WlanSetProfile, L2_NOTIFICATION_DATA, WLAN_CONNECTION_ATTRIBUTES, WLAN_CONNECTION_PARAMETERS, WLAN_INTF_OPCODE, WLAN_NOTIFICATION_SOURCES, WLAN_OPCODE_VALUE_TYPE};
use windows::Win32::{Foundation::HANDLE, NetworkManagement::WiFi::WLAN_PROFILE_INFO_LIST};
use windows::core::{GUID, PCWSTR, PWSTR};

use crate::windows_wlan::check_win32;

pub enum WlanError {
    Error(String),
    Win32Error(WIN32_ERROR)
}

pub fn register_notification
(
    client_handle: HANDLE, 
    source: WLAN_NOTIFICATION_SOURCES, 
    ignore_dupes: bool, 
    callback: Option<unsafe extern "system" fn(*mut L2_NOTIFICATION_DATA, *mut c_void)>,
) {
    let result = unsafe {
        WlanRegisterNotification(client_handle, source, ignore_dupes, callback, None, None, None)
    };

    if let Err(error) = check_win32(result) {
        godot_error!("[WLAN] Failed To Register Notification Callback: {:?}", error);
        return;
    }
}

pub fn scan(client_handle: HANDLE, interface_guid: &GUID) {
    let result = unsafe {
        WlanScan(client_handle, interface_guid, None, None, None)
    };

    if let Err(error) = check_win32(result) {
        godot_error!("[WLAN] Request Scan Failed: {:?}", error);
    } else {
        godot_print!("[WLAN] Scan Request Ok");
    }
}

pub fn query_interface<'a>(
    client_handle: HANDLE, 
    interface_guid: &'a GUID, 
    opcode: WLAN_INTF_OPCODE, 
    size: &mut u32, 
    opcode_type: &mut WLAN_OPCODE_VALUE_TYPE
) -> Result<Option<&'a WLAN_CONNECTION_ATTRIBUTES>, WlanError> {
    let mut data_ptr: *mut c_void = null_mut();
    
    let result = unsafe {
        WlanQueryInterface(client_handle, interface_guid, opcode, None, size, &mut data_ptr, Some(opcode_type))
    };

    if result != 0 {
        return Err(WlanError::Win32Error(WIN32_ERROR(result)));
    }

    unsafe {
        if data_ptr.is_null() {
            return Err(WlanError::Error("[WLAN] Failed To Query Interface. Data Pointer Was Null.".to_string()));
        } else {
            return Ok(Some(&*(data_ptr as *const WLAN_CONNECTION_ATTRIBUTES)));
        }
    }
}

pub fn get_profile_list(client_handle: HANDLE, interface_guid: &GUID) -> Result<ManuallyDrop<Box<WLAN_PROFILE_INFO_LIST>>, WIN32_ERROR> {
    let mut list_ptr: *mut WLAN_PROFILE_INFO_LIST = null_mut();

    let result = unsafe {
        WlanGetProfileList
        (
            client_handle, 
            interface_guid, 
            None, 
            &mut list_ptr
        )
    };

    if result != 0 {
        return Err(WIN32_ERROR(result));
    }

    let box_ptr = unsafe {
        ManuallyDrop::new(Box::from_raw(list_ptr))
    };

    Ok(box_ptr)
}

pub fn delete_profile(client_handle: HANDLE, interface_guid: &GUID, profile_name: &U16CString) {
    let name_ptr = PCWSTR::from_raw(profile_name.as_ptr());
    let result = unsafe {
        WlanDeleteProfile
        (
            client_handle, 
            interface_guid, 
            name_ptr, 
            None
        )
    };

    match check_win32(result) {
        Ok(_) => godot_print!("[WLAN] Deleted Profile"),
        Err(error) => {
            godot_error!("[WLAN] Failed To Delete Profile: {:?}", error);
            return;
        },
    }
}

pub fn get_profile(client_handle: HANDLE, interface_guid: &GUID, profile_name: &U16CString) -> Result<U16String, WIN32_ERROR> {
    let mut xml_u16string = U16CString::new();

    let profile_ptr = PCWSTR::from_raw(profile_name.as_ptr());
    let mut xml_ptr = PWSTR::from_raw(xml_u16string.as_mut_ptr());

    let result = unsafe {
        WlanGetProfile
        (
            client_handle, 
            interface_guid, 
            profile_ptr, 
            None, 
            &mut xml_ptr, 
            None, 
            None
        )
    };

    if result != 0 {
        return Err(WIN32_ERROR(result));
    }

    let xml_string = xml_u16string.to_ustring();
    Ok(xml_string)
}

pub fn set_profile(client_handle: HANDLE, interface_guid: &GUID, profile: &U16CString, overwrite: bool) -> Result<(), WIN32_ERROR> {
    let mut error_code = 0u32;
    let mut reason_buffer = [0u16; 512];

    let result = unsafe {
        WlanSetProfile
        (
            client_handle, 
            interface_guid, 
            0, 
            PCWSTR::from_raw(profile.as_ptr()), 
            PCWSTR::null(), 
            overwrite, 
            None, 
            &mut error_code
        )
    };

    if result != 0 {
        let u16_len = reason_buffer.iter().position(|&c| c == 0).unwrap_or(reason_buffer.len());
        let u16_string = String::from_utf16_lossy(&reason_buffer[..u16_len]);
        godot_print!("[WLAN] Set Profile Reason Code: {}", u16_string);
        
        return Err(WIN32_ERROR(result));
    }

    let reason_result = unsafe {
        WlanReasonCodeToString(error_code, &mut reason_buffer, None)
    };

    if reason_result != 0 {
        return Err(WIN32_ERROR(reason_result));
    }

    let u16_len = reason_buffer.iter().position(|&c| c == 0).unwrap_or(reason_buffer.len());
    let u16_string = String::from_utf16_lossy(&reason_buffer[..u16_len]);
    godot_print!("[WLAN] Set Profile Reason Code: {}", u16_string);

    Ok(())
}

pub fn connect(client_handle: HANDLE, interface_guid: &GUID, connection_params: &WLAN_CONNECTION_PARAMETERS) -> Result<(), WIN32_ERROR> {
    let result = unsafe {
        WlanConnect
        (
            client_handle, 
            interface_guid, 
            connection_params, 
            None
        )
    };

    if result != 0 {
        return Err(WIN32_ERROR(result));
    }

    Ok(())
}

pub fn disconnect(client_handle: HANDLE, interface_guid: &GUID) {
    let result = unsafe {
        WlanDisconnect(client_handle, interface_guid, None)
    };

    if let Err(error) = check_win32(result) {
        godot_error!("[WLAN] Failed To Disconnect From Network {:?}", error);
    } else {
        godot_print!("[WLAN] Disconnected From Network");
    }
}