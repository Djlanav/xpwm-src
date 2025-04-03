use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, LazyLock, Mutex};

use crate::wlan_enums::ConnectionNotifcation;

pub static CONNECTION_NOTIFICATION_CHANNEL: LazyLock<Arc<Mutex<(Sender<ConnectionNotifcation>, Receiver<ConnectionNotifcation>)>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(channel()))
});

#[cfg(debug_assertions)]
pub fn save_xml_to_disk(ssid: &str) {
    use godot::global::{godot_error, godot_print};

    use crate::profile_management::generate_network_profile_xml;
    use crate::wlan_enums::{EncryptionAlgorithm, NetworkSecurity};
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    let xml_data = generate_network_profile_xml(ssid, "LOOKATMEEE", &EncryptionAlgorithm::AES, &NetworkSecurity::WPA2PSK);

    let path = Path::new("debug_profiles").join(format!("{}_profile.xml", ssid));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap_or_else(|e| {
            godot_error!("Failed to create directory for debug profiles: {:?}", e);
        });
    }

    match File::create(&path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(xml_data.as_bytes()) {
                godot_error!("Failed to write profile to disk: {:?}", e);
            } else {
                godot_print!("Saved profile to: {:?}", path);
            }
        }
        Err(e) => {
            godot_error!("Failed to create profile file: {:?}", e);
        }
    }
}