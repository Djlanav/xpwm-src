use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, LazyLock, Mutex};

use crate::wlan_enums::ConnectionNotifcation;

pub static CONNECTION_NOTIFICATION_CHANNEL: LazyLock<Arc<Mutex<(Sender<ConnectionNotifcation>, Receiver<ConnectionNotifcation>)>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(channel()))
});