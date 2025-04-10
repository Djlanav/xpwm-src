pub mod wlan;
pub mod enums;

use enums::MessageBoxResult;
use godot::prelude::*;
use widestring::U16CString;
use windows::{core::PCWSTR, Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONWARNING, MB_OK, MB_YESNO}};

#[derive(GodotClass)]
#[class(init, base=Object)]
pub struct Win32API {
    base: Base<Object>
}

#[godot_api]
impl Win32API {
    #[func]
    fn show_error_message_box(&self, title: String, message: String) {
        let (title_u16, message_u16) = match create_double_u16cstring(&title, &message) {
            Some(tuple) => tuple,
            None => return,
        };

        unsafe {
            MessageBoxW(
                None, 
                PCWSTR::from_raw(message_u16.as_ptr()), 
                PCWSTR::from_raw(title_u16.as_ptr()), 
                MB_OK | MB_ICONERROR);
        }
    }

    #[func]
    fn show_info_message_box(&self, title: String, message: String) {
        let (title_u16, message_u16) = match create_double_u16cstring(&title, &message) {
            Some(tuple) => tuple,
            None => return,
        };

        unsafe {
            MessageBoxW(
                None, 
                PCWSTR::from_raw(message_u16.as_ptr()), 
                PCWSTR::from_raw(title_u16.as_ptr()), 
                MB_OK | MB_ICONINFORMATION);       
        }
    }

    #[func]
    fn show_yes_no_warning(&self, title: String, message: String) -> bool {
        let (title_u16, message_u16) = match create_double_u16cstring(&title, &message) {
            Some(tuple) => tuple,
            None => {
                godot_error!("[SYSTEM] Failed To Convert To U16CString");
                return false;
            }
        };

        unsafe {
            let result = MessageBoxW
            (
                None, 
                PCWSTR::from_raw(message_u16.as_ptr()), 
                PCWSTR::from_raw(title_u16.as_ptr()), 
                MB_YESNO | MB_ICONWARNING
            );

            let new_result = MessageBoxResult::convert(result);
            match new_result {
                MessageBoxResult::Yes => true,
                MessageBoxResult::No => false,
                _ => false
            }
        }
    }
}

pub fn convert_string_to_u16cstring(string: &String) -> Option<U16CString> {
    let string_u16 = match U16CString::from_str(string.as_str()) {
        Ok(u16_cstring) => u16_cstring,
        Err(error) => {
            godot_error!("[SYSTEM] Failed To Convert Title To Wide String. Error: {}", error);
            return None;
        },
    };

    Some(string_u16)
}

pub fn convert_u16_slice_to_u16cstring(u16_slice: &[u16]) -> Option<U16CString> {
    let end = u16_slice.iter().position(|&c| c == 0).unwrap_or(u16_slice.len());
    let slice = &u16_slice[..end];

    let u16_cstring = match U16CString::from_vec(slice) {
        Ok(cstring) => cstring,
        Err(error) =>  {
            godot_error!("[SYSTEM] Failed To Convert String Slice To U16CString: {:?}", error);
            return None;
        },
    };

    Some(u16_cstring)
}

pub fn convert_u16_slice_to_string(u16_slice: &[u16]) -> String {
    let end = u16_slice.iter().position(|&c| c == 0).unwrap_or(u16_slice.len());
    let rust_string = String::from_utf16_lossy(&u16_slice[..end]);

    rust_string
}

// pub fn convert_str_to_u16cstring(str: &str) -> Option<U16CString> {
//     let string_u16 = match U16CString::from_str(str) {
//         Ok(u16_cstring) => u16_cstring,
//         Err(error) => {
//             godot_error!("[SYSTEM] Failed To Convert Title To Wide String. Error: {}", error);
//             return None;
//         },
//     };

//     Some(string_u16)
// }

pub fn create_double_u16cstring(string1: &String, string2: &String) -> Option<(U16CString, U16CString)> {
    let string1_u16 = match convert_string_to_u16cstring(&string1) {
        Some(string1_u16) => string1_u16,
        None => return None,
    };

    let string2_u16 = match convert_string_to_u16cstring(&string2) {
        Some(string2_u16) => string2_u16,
        None => return None
    };

    Some((string1_u16, string2_u16))
}