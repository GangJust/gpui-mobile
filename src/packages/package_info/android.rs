use super::PackageInfo;
use crate::android::jni_helpers::JniEnv;

pub fn get_package_info() -> Result<PackageInfo, String> {
    let jni = JniEnv::obtain()?;
    let activity = jni.activity();

    unsafe {
        // activity.getPackageName() → String
        let package_name_jstr = jni.call_string_method(
            activity,
            b"getPackageName\0",
            b"()Ljava/lang/String;\0",
        );

        // activity.getPackageManager() → PackageManager
        let activity_cls = jni.find_class(b"android/app/Activity\0");
        if activity_cls.is_null() {
            return Err("FindClass Activity failed".into());
        }

        let get_pm = jni.get_method_id(
            activity_cls,
            b"getPackageManager\0",
            b"()Landroid/content/pm/PackageManager;\0",
        );
        jni.delete_local_ref(activity_cls);
        if get_pm.is_null() {
            return Err("getPackageManager method not found".into());
        }
        let pm = jni.call_object_method(activity, get_pm, &[]);
        if pm.is_null() {
            return Err("getPackageManager returned null".into());
        }

        // pm.getPackageInfo(packageName, 0) → PackageInfo
        let pm_cls = jni.find_class(b"android/content/pm/PackageManager\0");
        if pm_cls.is_null() {
            jni.delete_local_ref(pm);
            return Err("FindClass PackageManager failed".into());
        }
        let get_pi = jni.get_method_id(
            pm_cls,
            b"getPackageInfo\0",
            b"(Ljava/lang/String;I)Landroid/content/pm/PackageInfo;\0",
        );
        jni.delete_local_ref(pm_cls);
        if get_pi.is_null() {
            jni.delete_local_ref(pm);
            return Err("getPackageInfo method not found".into());
        }

        let pkg_name_jstr = jni.new_string_utf(&package_name_jstr);
        let pkg_info = jni.call_object_method(
            pm,
            get_pi,
            &[pkg_name_jstr as i64, 0i64],
        );
        jni.delete_local_ref(pkg_name_jstr);
        if pkg_info.is_null() {
            jni.delete_local_ref(pm);
            return Err("getPackageInfo returned null".into());
        }

        // Read PackageInfo fields
        let pi_cls = jni.find_class(b"android/content/pm/PackageInfo\0");
        if pi_cls.is_null() {
            jni.delete_local_ref(pkg_info);
            jni.delete_local_ref(pm);
            return Err("FindClass PackageInfo failed".into());
        }

        // versionName: String
        let vn_field = jni.get_field_id(pi_cls, b"versionName\0", b"Ljava/lang/String;\0");
        let version = if !vn_field.is_null() {
            let vn = jni.get_object_field(pkg_info, vn_field);
            let s = jni.get_string(vn);
            jni.delete_local_ref(vn);
            s
        } else {
            String::new()
        };

        // versionCode: int
        let vc_field = jni.get_field_id(pi_cls, b"versionCode\0", b"I\0");
        let build_number = if !vc_field.is_null() {
            jni.get_int_field_value(pkg_info, vc_field).to_string()
        } else {
            String::new()
        };

        // applicationInfo → getApplicationLabel
        let ai_field = jni.get_field_id(
            pi_cls,
            b"applicationInfo\0",
            b"Landroid/content/pm/ApplicationInfo;\0",
        );
        let app_name = if !ai_field.is_null() {
            let app_info = jni.get_object_field(pkg_info, ai_field);
            if !app_info.is_null() {
                let pm_cls2 = jni.get_object_class(pm);
                let get_label = jni.get_method_id(
                    pm_cls2,
                    b"getApplicationLabel\0",
                    b"(Landroid/content/pm/ApplicationInfo;)Ljava/lang/CharSequence;\0",
                );
                jni.delete_local_ref(pm_cls2);

                let label = if !get_label.is_null() {
                    let cs = jni.call_object_method(pm, get_label, &[app_info as i64]);
                    if !cs.is_null() {
                        let s = jni.call_string_method(cs, b"toString\0", b"()Ljava/lang/String;\0");
                        jni.delete_local_ref(cs);
                        s
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                jni.delete_local_ref(app_info);
                label
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        jni.delete_local_ref(pi_cls);
        jni.delete_local_ref(pkg_info);
        jni.delete_local_ref(pm);

        let package_name = package_name_jstr;

        Ok(PackageInfo {
            app_name,
            package_name,
            version,
            build_number,
        })
    }
}
