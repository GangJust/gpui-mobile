use super::HapticFeedback;
use crate::android::jni_helpers::JniEnv;

pub fn vibrate(duration_ms: u32) -> Result<(), String> {
    let jni = JniEnv::obtain()?;
    let activity = jni.activity();

    unsafe {
        let vibrator = get_vibrator_service(&jni, activity)?;

        // Try VibrationEffect.createOneShot (API 26+)
        let ve_cls = jni.find_class(b"android/os/VibrationEffect\0");
        if !ve_cls.is_null() {
            let create = jni.get_static_method_id(
                ve_cls,
                b"createOneShot\0",
                b"(JI)Landroid/os/VibrationEffect;\0",
            );
            if !create.is_null() {
                let effect = jni.call_static_object_method(
                    ve_cls,
                    create,
                    &[duration_ms as i64, -1i64], // DEFAULT_AMPLITUDE = -1
                );
                if !effect.is_null() {
                    let vib_cls = jni.get_object_class(vibrator);
                    let vibrate_method = jni.get_method_id(
                        vib_cls,
                        b"vibrate\0",
                        b"(Landroid/os/VibrationEffect;)V\0",
                    );
                    jni.delete_local_ref(vib_cls);
                    if !vibrate_method.is_null() {
                        jni.call_void_method(vibrator, vibrate_method, &[effect as i64]);
                    }
                    jni.delete_local_ref(effect);
                    jni.delete_local_ref(ve_cls);
                    jni.delete_local_ref(vibrator);
                    return Ok(());
                }
            }
            jni.delete_local_ref(ve_cls);
        }

        // Fallback: vibrator.vibrate(long) for older APIs
        let vib_cls = jni.get_object_class(vibrator);
        let vibrate_method = jni.get_method_id(vib_cls, b"vibrate\0", b"(J)V\0");
        jni.delete_local_ref(vib_cls);
        if !vibrate_method.is_null() {
            jni.call_void_method(vibrator, vibrate_method, &[duration_ms as i64]);
        }
        jni.delete_local_ref(vibrator);
    }
    Ok(())
}

pub fn haptic_feedback(feedback: HapticFeedback) -> Result<(), String> {
    // Map to Android HapticFeedbackConstants
    let constant: i32 = match feedback {
        HapticFeedback::Light => 1,     // VIRTUAL_KEY
        HapticFeedback::Medium => 1,    // VIRTUAL_KEY
        HapticFeedback::Heavy => 0,     // LONG_PRESS
        HapticFeedback::Selection => 3, // KEYBOARD_TAP
        HapticFeedback::Success => 1,   // VIRTUAL_KEY
        HapticFeedback::Warning => 0,   // LONG_PRESS
        HapticFeedback::Error => 0,     // LONG_PRESS
    };

    let jni = JniEnv::obtain()?;
    let activity = jni.activity();

    unsafe {
        // activity.getWindow().getDecorView().performHapticFeedback(constant)
        let activity_cls = jni.find_class(b"android/app/Activity\0");
        if activity_cls.is_null() { return Err("FindClass Activity failed".into()); }

        let get_window = jni.get_method_id(
            activity_cls, b"getWindow\0", b"()Landroid/view/Window;\0",
        );
        jni.delete_local_ref(activity_cls);
        if get_window.is_null() { return Err("getWindow not found".into()); }

        let window = jni.call_object_method(activity, get_window, &[]);
        if window.is_null() { return Err("getWindow returned null".into()); }

        let window_cls = jni.get_object_class(window);
        let get_decor = jni.get_method_id(
            window_cls, b"getDecorView\0", b"()Landroid/view/View;\0",
        );
        jni.delete_local_ref(window_cls);

        if get_decor.is_null() {
            jni.delete_local_ref(window);
            return Err("getDecorView not found".into());
        }

        let decor = jni.call_object_method(window, get_decor, &[]);
        jni.delete_local_ref(window);
        if decor.is_null() { return Err("getDecorView returned null".into()); }

        let view_cls = jni.get_object_class(decor);
        let perform = jni.get_method_id(
            view_cls, b"performHapticFeedback\0", b"(I)Z\0",
        );
        jni.delete_local_ref(view_cls);

        if !perform.is_null() {
            jni.call_boolean_method(decor, perform, &[constant as i64]);
        }
        jni.delete_local_ref(decor);
    }
    Ok(())
}

pub fn can_vibrate() -> bool {
    let jni = match JniEnv::obtain() {
        Ok(j) => j,
        Err(_) => return false,
    };
    let activity = jni.activity();

    unsafe {
        let vibrator = match get_vibrator_service(&jni, activity) {
            Ok(v) => v,
            Err(_) => return false,
        };

        let vib_cls = jni.get_object_class(vibrator);
        let has_vibrator = jni.get_method_id(vib_cls, b"hasVibrator\0", b"()Z\0");
        jni.delete_local_ref(vib_cls);

        let result = if !has_vibrator.is_null() {
            jni.call_boolean_method(vibrator, has_vibrator, &[])
        } else {
            false
        };
        jni.delete_local_ref(vibrator);
        result
    }
}

unsafe fn get_vibrator_service(
    jni: &JniEnv, activity: *mut std::ffi::c_void,
) -> Result<*mut std::ffi::c_void, String> {
    let service_name = jni.new_string_utf("vibrator");
    let activity_cls = jni.find_class(b"android/app/Activity\0");
    if activity_cls.is_null() {
        jni.delete_local_ref(service_name);
        return Err("FindClass Activity failed".into());
    }
    let get_service = jni.get_method_id(
        activity_cls,
        b"getSystemService\0",
        b"(Ljava/lang/String;)Ljava/lang/Object;\0",
    );
    jni.delete_local_ref(activity_cls);

    if get_service.is_null() {
        jni.delete_local_ref(service_name);
        return Err("getSystemService not found".into());
    }

    let vibrator = jni.call_object_method(activity, get_service, &[service_name as i64]);
    jni.delete_local_ref(service_name);

    if vibrator.is_null() {
        return Err("Vibrator service not available".into());
    }
    Ok(vibrator)
}
