mod networking;
mod utils;
mod windows_api;
mod wlan_godot;
mod wlan_enums;
mod callbacks;
mod globals;
mod profile_management;

use godot::{classes::Engine, prelude::*};
use windows_api::Win32API;
use wlan_godot::WlanAPI;

struct EWindowsAPI;

#[gdextension]
unsafe impl ExtensionLibrary for EWindowsAPI {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            if !Engine::singleton().has_singleton("WlanAPI") {
                godot_print!("[WLAN] Initializing API");
                Engine::singleton().register_singleton("WlanAPI", &WlanAPI::new_alloc());
                godot_print!("[WLAN] API Initialized");
            }

            if !Engine::singleton().has_singleton("Win32API") {
                godot_print!("[WIN32] Initializing API");
                Engine::singleton().register_singleton("Win32API", &Win32API::new_alloc());
                godot_print!("[WIN32] API Initialized");
            }
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            let mut engine = Engine::singleton();
            let wlan_api = "WlanAPI";
            let win32_api = "Win32API";

            if let Some(engine_singleton) = engine.get_singleton(wlan_api) {
                if engine.has_method("close_wlan_handle") {
                    engine.call("close_wlan_handle", &[]);
                }

                engine.unregister_singleton(wlan_api);

                engine_singleton.free();
            } else {
                godot_error!("[CORE] Failed To Get Singleton: {}", wlan_api);
                panic!("Failed To Get Singleton");
            }

            if let Some(engine_singleton) = engine.get_singleton(win32_api) {
                engine.unregister_singleton(win32_api);
                engine_singleton.free();
            } else {
                godot_error!("[CORE] Failed To Get Singleton: {}", win32_api);
                panic!("Failed To Get Singleton");
            }
        }
    }
}
