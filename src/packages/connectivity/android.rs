use super::ConnectivityStatus;
use crate::android::jni_helpers::JniEnv;

pub fn check_connectivity() -> ConnectivityStatus {
    let jni = match JniEnv::obtain() {
        Ok(j) => j,
        Err(_) => return ConnectivityStatus::None,
    };
    let activity = jni.activity();

    unsafe {
        // context.getSystemService("connectivity") → ConnectivityManager
        let service_name = jni.new_string_utf("connectivity");
        let activity_cls = jni.find_class(b"android/app/Activity\0");
        if activity_cls.is_null() {
            jni.delete_local_ref(service_name);
            return ConnectivityStatus::None;
        }
        let get_service = jni.get_method_id(
            activity_cls,
            b"getSystemService\0",
            b"(Ljava/lang/String;)Ljava/lang/Object;\0",
        );
        jni.delete_local_ref(activity_cls);

        if get_service.is_null() {
            jni.delete_local_ref(service_name);
            return ConnectivityStatus::None;
        }

        let cm = jni.call_object_method(activity, get_service, &[service_name as i64]);
        jni.delete_local_ref(service_name);
        if cm.is_null() {
            return ConnectivityStatus::None;
        }

        // cm.getActiveNetworkInfo() → NetworkInfo
        let cm_cls = jni.get_object_class(cm);
        let get_info = jni.get_method_id(
            cm_cls,
            b"getActiveNetworkInfo\0",
            b"()Landroid/net/NetworkInfo;\0",
        );
        jni.delete_local_ref(cm_cls);

        if get_info.is_null() {
            jni.delete_local_ref(cm);
            return ConnectivityStatus::None;
        }

        let net_info = jni.call_object_method(cm, get_info, &[]);
        jni.delete_local_ref(cm);
        if net_info.is_null() {
            return ConnectivityStatus::None;
        }

        // networkInfo.isConnected()
        let ni_cls = jni.get_object_class(net_info);
        let is_connected_method = jni.get_method_id(ni_cls, b"isConnected\0", b"()Z\0");
        if is_connected_method.is_null() {
            jni.delete_local_ref(ni_cls);
            jni.delete_local_ref(net_info);
            return ConnectivityStatus::None;
        }
        let connected = jni.call_boolean_method(net_info, is_connected_method, &[]);
        if !connected {
            jni.delete_local_ref(ni_cls);
            jni.delete_local_ref(net_info);
            return ConnectivityStatus::None;
        }

        // networkInfo.getType()
        let get_type = jni.get_method_id(ni_cls, b"getType\0", b"()I\0");
        jni.delete_local_ref(ni_cls);

        let status = if !get_type.is_null() {
            let net_type = jni.call_int_method(net_info, get_type, &[]);
            match net_type {
                1 => ConnectivityStatus::Wifi,     // TYPE_WIFI
                0 => ConnectivityStatus::Cellular,  // TYPE_MOBILE
                _ => ConnectivityStatus::Wifi,      // Ethernet etc. treated as Wifi
            }
        } else {
            ConnectivityStatus::None
        };

        jni.delete_local_ref(net_info);
        status
    }
}
