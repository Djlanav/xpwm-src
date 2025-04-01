use crate::wlan_enums::{ConnectionNotifcation, convert_connection_notification};
use crate::globals::CONNECTION_NOTIFICATION_CHANNEL;
use std::ffi::c_void;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::MutexGuard;
use std::sync::TryLockError::{Poisoned, WouldBlock};
use windows::Win32::NetworkManagement::WiFi::*;

pub extern "system" fn wlan_acm_notification_callback(notification: *mut L2_NOTIFICATION_DATA, _context: *mut c_void) {
    let guard = match CONNECTION_NOTIFICATION_CHANNEL.try_lock() {
        Ok(guard) => guard,
        Err(error) => {
            match error {
                Poisoned(poison_error) => poison_error.into_inner(),
                WouldBlock => return
            }
        },
    };
    
    unsafe {
        let notif = &*notification;
        let acm_notif = WLAN_NOTIFICATION_ACM(notif.NotificationCode as i32);
        
        let converted_notif = convert_connection_notification(acm_notif);
        send_notification_data(&guard, converted_notif);
    }
}

fn send_notification_data(
    guard: &MutexGuard<'_, (Sender<ConnectionNotifcation>, Receiver<ConnectionNotifcation>)>, 
    data: ConnectionNotifcation) 
    {
    let sender = guard.0.clone();
    sender.send(data).unwrap();
}