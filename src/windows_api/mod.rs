pub mod wlan;

use godot::prelude::*;
use widestring::U16CString;
use windows::{core::PCWSTR, Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION, MB_OK}};

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
}

fn convert_string_to_u16cstring(string: &String) -> Option<U16CString> {
    let string_u16 = match U16CString::from_str(string.as_str()) {
        Ok(u16_cstring) => u16_cstring,
        Err(error) => {
            godot_error!("[SYSTEM] Failed To Convert Title To Wide String. Error: {}", error);
            return None;
        },
    };

    Some(string_u16)
}

fn create_double_u16cstring(string1: &String, string2: &String) -> Option<(U16CString, U16CString)> {
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