mod windows_wlan;
mod wlan_godot;
mod wlan_enums;
mod callbacks;
mod globals;

use godot::prelude::*;

struct EWindowsAPI;

#[gdextension]
unsafe impl ExtensionLibrary for EWindowsAPI {}
