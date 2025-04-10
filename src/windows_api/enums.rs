use windows::Win32::UI::WindowsAndMessaging::*;

#[allow(dead_code)]
pub enum MessageBoxResult {
    Yes,
    No,
    Ok,
    Cancel,
    Unknown
}

impl MessageBoxResult {
    pub fn convert(win32_result: MESSAGEBOX_RESULT) -> Self {
        match win32_result {
            IDYES => Self::Yes,
            IDNO => Self::No,
            _ => Self::Unknown
        }
    }
}