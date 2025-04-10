use xmlwriter::XmlWriter;

use crate::wlan_enums::{EncryptionAlgorithm, NetworkSecurity};

pub fn generate_network_profile_xml(
    ssid: &str, 
    password: &str, 
    encryption: &EncryptionAlgorithm,
    security: &NetworkSecurity) -> String
{
    let xml_options = xmlwriter::Options {
        use_single_quote: true,
        indent: xmlwriter::Indent::Spaces(4),
        attributes_indent: xmlwriter::Indent::None,
    };

    let mut writer = XmlWriter::new(xml_options);

    let encryption_string = encryption.convert_to_string();
    let security_string = security.convert_to_string();
    let ssid_hex = ssid.as_bytes().iter().map(|b| format!("{:02X}", b)).collect::<String>();

    let is_open = match security {
        NetworkSecurity::Open => true,
        _ => false
    };

    writer.start_element("WLANProfile");
    writer.write_attribute("xmlns", "http://www.microsoft.com/networking/WLAN/profile/v1");

    write_element_preserve_white_spaces(&mut writer, "name", ssid);
    writer.start_element("SSIDConfig");
        writer.start_element("SSID");
            write_element_preserve_white_spaces(&mut writer, "hex", ssid_hex.as_str());
            write_element_preserve_white_spaces(&mut writer, "name", ssid);
        writer.end_element(); // </SSID>
        write_element_preserve_white_spaces(&mut writer, "nonBroadcast", "false");
    writer.end_element(); // <SSIDConfig>
    write_element_preserve_white_spaces(&mut writer, "connectionType", "ESS");
    write_element_preserve_white_spaces(&mut writer, "connectionMode", "manual");
    write_element_preserve_white_spaces(&mut writer, "autoSwitch", "false");
    writer.start_element("MSM");
        writer.start_element("security");
            writer.start_element("authEncryption");
                write_element_preserve_white_spaces(&mut writer, "authentication", &security_string);
                write_element_preserve_white_spaces(&mut writer, "encryption", &encryption_string);
                write_element_preserve_white_spaces(&mut writer, "useOneX", "false");
            writer.end_element(); // </authEncryption>
            if !is_open {
                write_shared_key_element(&mut writer, password);
            }
        writer.end_element(); // </security>
    writer.end_element(); // </MSM>
    
    writer.end_document() // </WLANProfile>

}

fn write_element_preserve_white_spaces(writer: &mut XmlWriter, element_name: &str, text: &str) {
    writer.start_element(element_name);
        writer.set_preserve_whitespaces(true);
        writer.write_text(text);
    writer.end_element();
    writer.set_preserve_whitespaces(false);
}

fn write_shared_key_element(writer: &mut XmlWriter, password: &str) {
    writer.start_element("sharedKey");
            write_element_preserve_white_spaces(writer, "keyType", "passPhrase");
            write_element_preserve_white_spaces(writer, "protected", "false");
            write_element_preserve_white_spaces(writer, "keyMaterial", password);
    writer.end_element(); // </sharedKey>
}