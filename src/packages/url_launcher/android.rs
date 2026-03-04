use crate::android::jni_helpers::JniEnv;

pub fn launch_url(url: &str) -> Result<bool, String> {
    let jni = JniEnv::obtain()?;
    let activity = jni.activity();

    unsafe {
        let intent = create_view_intent(&jni, url)?;

        // activity.startActivity(intent)
        let activity_cls = jni.find_class(b"android/app/Activity\0");
        if activity_cls.is_null() {
            jni.delete_local_ref(intent);
            return Err("FindClass Activity failed".into());
        }
        let start_activity = jni.get_method_id(
            activity_cls,
            b"startActivity\0",
            b"(Landroid/content/Intent;)V\0",
        );
        jni.delete_local_ref(activity_cls);

        if start_activity.is_null() {
            jni.delete_local_ref(intent);
            return Err("startActivity method not found".into());
        }

        jni.call_void_method(activity, start_activity, &[intent as i64]);
        let had_exception = jni.exception_check();
        if had_exception {
            jni.clear_exception();
        }

        jni.delete_local_ref(intent);
        Ok(!had_exception)
    }
}

pub fn can_launch_url(url: &str) -> Result<bool, String> {
    let jni = JniEnv::obtain()?;
    let activity = jni.activity();

    unsafe {
        let intent = create_view_intent(&jni, url)?;

        // activity.getPackageManager()
        let activity_cls = jni.find_class(b"android/app/Activity\0");
        if activity_cls.is_null() {
            jni.delete_local_ref(intent);
            return Err("FindClass Activity failed".into());
        }
        let get_pm = jni.get_method_id(
            activity_cls,
            b"getPackageManager\0",
            b"()Landroid/content/pm/PackageManager;\0",
        );
        jni.delete_local_ref(activity_cls);

        if get_pm.is_null() {
            jni.delete_local_ref(intent);
            return Err("getPackageManager method not found".into());
        }

        let pm = jni.call_object_method(activity, get_pm, &[]);
        if pm.is_null() {
            jni.delete_local_ref(intent);
            return Err("getPackageManager returned null".into());
        }

        // pm.resolveActivity(intent, 0)
        let pm_cls = jni.get_object_class(pm);
        let resolve = jni.get_method_id(
            pm_cls,
            b"resolveActivity\0",
            b"(Landroid/content/Intent;I)Landroid/content/pm/ResolveInfo;\0",
        );
        jni.delete_local_ref(pm_cls);

        let can_resolve = if !resolve.is_null() {
            let resolved = jni.call_object_method(pm, resolve, &[intent as i64, 0i64]);
            let ok = !resolved.is_null();
            jni.delete_local_ref(resolved);
            ok
        } else {
            false
        };

        jni.delete_local_ref(pm);
        jni.delete_local_ref(intent);

        Ok(can_resolve)
    }
}

/// Create an Intent(ACTION_VIEW, Uri.parse(url)).
unsafe fn create_view_intent(jni: &JniEnv, url: &str) -> Result<*mut std::ffi::c_void, String> {
    // Uri.parse(url)
    let uri_cls = jni.find_class(b"android/net/Uri\0");
    if uri_cls.is_null() { return Err("FindClass Uri failed".into()); }

    let parse = jni.get_static_method_id(
        uri_cls,
        b"parse\0",
        b"(Ljava/lang/String;)Landroid/net/Uri;\0",
    );
    if parse.is_null() {
        jni.delete_local_ref(uri_cls);
        return Err("Uri.parse method not found".into());
    }

    let jurl = jni.new_string_utf(url);
    let uri = jni.call_static_object_method(uri_cls, parse, &[jurl as i64]);
    jni.delete_local_ref(jurl);
    jni.delete_local_ref(uri_cls);

    if uri.is_null() {
        return Err(format!("Uri.parse returned null for: {url}"));
    }

    // new Intent(ACTION_VIEW, uri)
    let intent_cls = jni.find_class(b"android/content/Intent\0");
    if intent_cls.is_null() {
        jni.delete_local_ref(uri);
        return Err("FindClass Intent failed".into());
    }

    let ctor = jni.get_method_id(
        intent_cls,
        b"<init>\0",
        b"(Ljava/lang/String;Landroid/net/Uri;)V\0",
    );
    if ctor.is_null() {
        jni.delete_local_ref(intent_cls);
        jni.delete_local_ref(uri);
        return Err("Intent constructor not found".into());
    }

    let action_view = jni.new_string_utf("android.intent.action.VIEW");
    let intent = jni.new_object(intent_cls, ctor, &[action_view as i64, uri as i64]);
    jni.delete_local_ref(action_view);
    jni.delete_local_ref(uri);
    jni.delete_local_ref(intent_cls);

    if intent.is_null() {
        return Err("Failed to create Intent".into());
    }

    Ok(intent)
}
