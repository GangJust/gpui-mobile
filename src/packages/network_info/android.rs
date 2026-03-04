use super::NetworkInfo;
use crate::android::jni_helpers::JniEnv;

pub fn get_network_info() -> Result<NetworkInfo, String> {
    let jni = JniEnv::obtain()?;
    let activity = jni.activity();
    let mut info = NetworkInfo::default();

    unsafe {
        // context.getSystemService("wifi") → WifiManager
        let service_name = jni.new_string_utf("wifi");
        let activity_cls = jni.find_class(b"android/app/Activity\0");
        if activity_cls.is_null() {
            jni.delete_local_ref(service_name);
            return Ok(info);
        }
        let get_service = jni.get_method_id(
            activity_cls,
            b"getSystemService\0",
            b"(Ljava/lang/String;)Ljava/lang/Object;\0",
        );
        jni.delete_local_ref(activity_cls);
        if get_service.is_null() {
            jni.delete_local_ref(service_name);
            return Ok(info);
        }

        let wifi_mgr = jni.call_object_method(activity, get_service, &[service_name as i64]);
        jni.delete_local_ref(service_name);
        if wifi_mgr.is_null() {
            return Ok(info);
        }

        // wifiManager.getConnectionInfo() → WifiInfo
        let wm_cls = jni.get_object_class(wifi_mgr);
        let get_conn = jni.get_method_id(
            wm_cls,
            b"getConnectionInfo\0",
            b"()Landroid/net/wifi/WifiInfo;\0",
        );
        jni.delete_local_ref(wm_cls);
        if get_conn.is_null() {
            jni.delete_local_ref(wifi_mgr);
            return Ok(info);
        }

        let wifi_info = jni.call_object_method(wifi_mgr, get_conn, &[]);
        jni.delete_local_ref(wifi_mgr);
        if wifi_info.is_null() {
            return Ok(info);
        }

        let wi_cls = jni.get_object_class(wifi_info);

        // SSID
        let get_ssid = jni.get_method_id(wi_cls, b"getSSID\0", b"()Ljava/lang/String;\0");
        if !get_ssid.is_null() {
            let ssid_jstr = jni.call_object_method(wifi_info, get_ssid, &[]);
            if !ssid_jstr.is_null() {
                let ssid = jni.get_string(ssid_jstr);
                jni.delete_local_ref(ssid_jstr);
                let ssid = ssid.trim_matches('"').to_string();
                if ssid != "<unknown ssid>" {
                    info.wifi_name = Some(ssid);
                }
            }
        }

        // BSSID
        let get_bssid = jni.get_method_id(wi_cls, b"getBSSID\0", b"()Ljava/lang/String;\0");
        if !get_bssid.is_null() {
            let bssid_jstr = jni.call_object_method(wifi_info, get_bssid, &[]);
            if !bssid_jstr.is_null() {
                let bssid = jni.get_string(bssid_jstr);
                jni.delete_local_ref(bssid_jstr);
                if bssid != "02:00:00:00:00:00" {
                    info.wifi_bssid = Some(bssid);
                }
            }
        }

        // IP Address
        let get_ip = jni.get_method_id(wi_cls, b"getIpAddress\0", b"()I\0");
        if !get_ip.is_null() {
            let ip = jni.call_int_method(wifi_info, get_ip, &[]);
            if ip != 0 {
                info.wifi_ip = Some(format!(
                    "{}.{}.{}.{}",
                    ip & 0xFF,
                    (ip >> 8) & 0xFF,
                    (ip >> 16) & 0xFF,
                    (ip >> 24) & 0xFF,
                ));
            }
        }

        jni.delete_local_ref(wi_cls);
        jni.delete_local_ref(wifi_info);
    }

    Ok(info)
}
