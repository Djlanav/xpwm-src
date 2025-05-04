use serde::Deserialize;
use wmi::{COMLibrary, WMIConnection};
use godot::prelude::*;

use super::NetworkManager;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NetworkAdapter {
    name: String,
    net_connection_status: Option<u16>,
    manufacturer: Option<String>,
    driver_name: Option<String>,
}

impl NetworkManager {
    pub fn query_adapters(&self) -> Option<Vec<NetworkAdapter>> {
        let com_con = match COMLibrary::new() {
            Ok(com) => com,
            Err(error) => {
                godot_error!("[SYSTEM] Failed To Establish COM Connection: {}", error);
                return None;
            },
        };
        let wmi_con = match WMIConnection::new(com_con.into()) {
            Ok(con) => con,
            Err(error) => {
                godot_error!("[SYSTEM] Failed To Establish WMI Connection: {}", error);
                return None;
            },
        };

        let query_result: Result<Vec<NetworkAdapter>, wmi::WMIError> = wmi_con.raw_query(
            "SELECT Name, Manufacturer, DriverName, NetConnectionStatus FROM Win32_NetworkAdapter WHERE NetConnectionStatus = 2"
        );

        let adapters = match query_result {
            Ok(adapters) => adapters,
            Err(error) => {
                godot_error!("[SYSTEM] Failed To Query Network Adapters: {}", error);
                return None;
            },
        };

        Some(adapters)
    }

    pub fn check_adapters() {
        
    }
}