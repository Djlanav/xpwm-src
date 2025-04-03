mod windows_wlan;
mod wlan_godot;
mod wlan_enums;
mod callbacks;
mod globals;
mod profile_management;

use godot::{classes::Engine, prelude::*};
use wlan_godot::WlanAPI;

struct EWindowsAPI;

#[gdextension]
unsafe impl ExtensionLibrary for EWindowsAPI {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            godot_print!("[WLAN] Initializing API");
            Engine::singleton().register_singleton("WlanAPI", &WlanAPI::new_alloc());
            godot_print!("[WLAN] API Initialized");
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            let mut engine = Engine::singleton();
            let singleton_name = "WlanAPI";

            if let Some(engine_singleton) = engine.get_singleton(singleton_name) {
                engine.call("close_wlan_handle", &[]);
                engine.unregister_singleton(singleton_name);

                engine_singleton.free();
            } else {
                godot_error!("[CORE] Failed To Get Singleton: {}", singleton_name);
                panic!("Failed To Get Singleton");
            }
        }
    }
}
