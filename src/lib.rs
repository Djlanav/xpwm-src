mod windows_wlan;
mod wlan_godot;
mod wlan_enums;

use godot::prelude::*;

struct EWindowsAPI;

#[gdextension]
unsafe impl ExtensionLibrary for EWindowsAPI {}
