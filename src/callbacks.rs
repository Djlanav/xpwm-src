use crate::wlan_enums::{convert_connection_notification, convert_msm_notification, ConnectionNotifcation};
use crate::globals::CONNECTION_NOTIFICATION_CHANNEL;
use std::ffi::c_void;
use std::ptr;
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
        let notif = if !notification.is_null() {
            &ptr::read(notification)
        } else {
            send_notification_data(&guard, ConnectionNotifcation::Error);
            return;
        };
        
        match notif.NotificationSource {
            WLAN_NOTIFICATION_SOURCE_ACM => {
                let notif_code = WLAN_NOTIFICATION_ACM(notif.NotificationCode as i32);
                let notif_enum = convert_connection_notification(notif_code);
                send_notification_data(&guard, notif_enum);
            }

            WLAN_NOTIFICATION_SOURCE_MSM => {
                if !notif.pData.is_null() {
                    let data_ref = ptr::read(notif.pData as *const u32);
                    let notif_code = notif.NotificationCode;

                    let notif_enum = convert_msm_notification(notif_code, data_ref);
                    send_notification_data(&guard, notif_enum);
                } else {
                    send_notification_data(&guard, ConnectionNotifcation::Error);
                }
            }

            _ => {}
        }
    }
}

fn send_notification_data(
    guard: &MutexGuard<'_, (Sender<ConnectionNotifcation>, Receiver<ConnectionNotifcation>)>, 
    data: ConnectionNotifcation) 
    {
    let sender = guard.0.clone();
    sender.send(data).unwrap();
}