use crate::android::jni_helpers::JniEnv;

pub struct AndroidSharedPreferences;

impl AndroidSharedPreferences {
    pub fn new() -> Self {
        Self
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        let jni = JniEnv::obtain().ok()?;
        let prefs = get_default_prefs(&jni)?;
        unsafe {
            let jkey = jni.new_string_utf(key);
            if jkey.is_null() { jni.delete_local_ref(prefs); return None; }

            let prefs_cls = jni.get_object_class(prefs);
            let method = jni.get_method_id(
                prefs_cls,
                b"getString\0",
                b"(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;\0",
            );
            jni.delete_local_ref(prefs_cls);

            if method.is_null() {
                jni.delete_local_ref(jkey);
                jni.delete_local_ref(prefs);
                return None;
            }

            let result = jni.call_object_method(prefs, method, &[jkey as i64, 0i64]); // null default
            jni.delete_local_ref(jkey);

            let s = if !result.is_null() {
                let v = jni.get_string(result);
                jni.delete_local_ref(result);
                Some(v)
            } else {
                None
            };

            jni.delete_local_ref(prefs);
            s
        }
    }

    pub fn set_string(&self, key: &str, value: &str) -> Result<(), String> {
        with_editor(|jni, editor| unsafe {
            let jkey = jni.new_string_utf(key);
            let jval = jni.new_string_utf(value);
            let cls = jni.get_object_class(editor);
            let method = jni.get_method_id(
                cls,
                b"putString\0",
                b"(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;\0",
            );
            jni.delete_local_ref(cls);
            if !method.is_null() {
                jni.call_object_method(editor, method, &[jkey as i64, jval as i64]);
            }
            jni.delete_local_ref(jkey);
            jni.delete_local_ref(jval);
        })
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        let jni = JniEnv::obtain().ok()?;
        let prefs = get_default_prefs(&jni)?;
        unsafe {
            if !self.contains_key_jni(&jni, prefs, key) {
                jni.delete_local_ref(prefs);
                return None;
            }
            let jkey = jni.new_string_utf(key);
            let prefs_cls = jni.get_object_class(prefs);
            let method = jni.get_method_id(prefs_cls, b"getLong\0", b"(Ljava/lang/String;J)J\0");
            jni.delete_local_ref(prefs_cls);
            let val = if !method.is_null() {
                Some(jni.call_long_method(prefs, method, &[jkey as i64, 0i64]))
            } else {
                None
            };
            jni.delete_local_ref(jkey);
            jni.delete_local_ref(prefs);
            val
        }
    }

    pub fn set_int(&self, key: &str, value: i64) -> Result<(), String> {
        with_editor(|jni, editor| unsafe {
            let jkey = jni.new_string_utf(key);
            let cls = jni.get_object_class(editor);
            let method = jni.get_method_id(
                cls,
                b"putLong\0",
                b"(Ljava/lang/String;J)Landroid/content/SharedPreferences$Editor;\0",
            );
            jni.delete_local_ref(cls);
            if !method.is_null() {
                jni.call_object_method(editor, method, &[jkey as i64, value]);
            }
            jni.delete_local_ref(jkey);
        })
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        let jni = JniEnv::obtain().ok()?;
        let prefs = get_default_prefs(&jni)?;
        unsafe {
            if !self.contains_key_jni(&jni, prefs, key) {
                jni.delete_local_ref(prefs);
                return None;
            }
            let jkey = jni.new_string_utf(key);
            let prefs_cls = jni.get_object_class(prefs);
            let method = jni.get_method_id(prefs_cls, b"getBoolean\0", b"(Ljava/lang/String;Z)Z\0");
            jni.delete_local_ref(prefs_cls);
            let val = if !method.is_null() {
                Some(jni.call_boolean_method(prefs, method, &[jkey as i64, 0i64]))
            } else {
                None
            };
            jni.delete_local_ref(jkey);
            jni.delete_local_ref(prefs);
            val
        }
    }

    pub fn set_bool(&self, key: &str, value: bool) -> Result<(), String> {
        with_editor(|jni, editor| unsafe {
            let jkey = jni.new_string_utf(key);
            let cls = jni.get_object_class(editor);
            let method = jni.get_method_id(
                cls,
                b"putBoolean\0",
                b"(Ljava/lang/String;Z)Landroid/content/SharedPreferences$Editor;\0",
            );
            jni.delete_local_ref(cls);
            if !method.is_null() {
                jni.call_boolean_method(editor, method, &[jkey as i64, value as i64]);
            }
            jni.delete_local_ref(jkey);
        })
    }

    pub fn remove(&self, key: &str) -> Result<(), String> {
        with_editor(|jni, editor| unsafe {
            let jkey = jni.new_string_utf(key);
            let cls = jni.get_object_class(editor);
            let method = jni.get_method_id(
                cls,
                b"remove\0",
                b"(Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;\0",
            );
            jni.delete_local_ref(cls);
            if !method.is_null() {
                jni.call_object_method(editor, method, &[jkey as i64]);
            }
            jni.delete_local_ref(jkey);
        })
    }

    pub fn clear(&self) -> Result<(), String> {
        with_editor(|jni, editor| unsafe {
            let cls = jni.get_object_class(editor);
            let method = jni.get_method_id(
                cls,
                b"clear\0",
                b"()Landroid/content/SharedPreferences$Editor;\0",
            );
            jni.delete_local_ref(cls);
            if !method.is_null() {
                jni.call_object_method(editor, method, &[]);
            }
        })
    }

    pub fn contains_key(&self, key: &str) -> bool {
        let jni = match JniEnv::obtain() {
            Ok(j) => j,
            Err(_) => return false,
        };
        let prefs = match get_default_prefs(&jni) {
            Some(p) => p,
            None => return false,
        };
        unsafe {
            let result = self.contains_key_jni(&jni, prefs, key);
            jni.delete_local_ref(prefs);
            result
        }
    }

    unsafe fn contains_key_jni(&self, jni: &JniEnv, prefs: *mut std::ffi::c_void, key: &str) -> bool {
        let jkey = jni.new_string_utf(key);
        let prefs_cls = jni.get_object_class(prefs);
        let method = jni.get_method_id(prefs_cls, b"contains\0", b"(Ljava/lang/String;)Z\0");
        jni.delete_local_ref(prefs_cls);
        let result = if !method.is_null() {
            jni.call_boolean_method(prefs, method, &[jkey as i64])
        } else {
            false
        };
        jni.delete_local_ref(jkey);
        result
    }
}

/// Get default SharedPreferences via PreferenceManager.
unsafe fn get_default_prefs(jni: &JniEnv) -> Option<*mut std::ffi::c_void> {
    let pm_cls = jni.find_class(b"android/preference/PreferenceManager\0");
    if pm_cls.is_null() { return None; }

    let method = jni.get_static_method_id(
        pm_cls,
        b"getDefaultSharedPreferences\0",
        b"(Landroid/content/Context;)Landroid/content/SharedPreferences;\0",
    );
    if method.is_null() {
        jni.delete_local_ref(pm_cls);
        return None;
    }

    let activity = jni.activity();
    let prefs = jni.call_static_object_method(pm_cls, method, &[activity as i64]);
    jni.delete_local_ref(pm_cls);
    if prefs.is_null() { None } else { Some(prefs) }
}

/// Get an editor, run the callback, then commit.
fn with_editor(f: impl FnOnce(&JniEnv, *mut std::ffi::c_void)) -> Result<(), String> {
    let jni = JniEnv::obtain()?;
    let prefs = unsafe { get_default_prefs(&jni) }
        .ok_or_else(|| "Failed to get SharedPreferences".to_string())?;

    unsafe {
        let prefs_cls = jni.get_object_class(prefs);
        let edit_method = jni.get_method_id(
            prefs_cls,
            b"edit\0",
            b"()Landroid/content/SharedPreferences$Editor;\0",
        );
        jni.delete_local_ref(prefs_cls);

        if edit_method.is_null() {
            jni.delete_local_ref(prefs);
            return Err("edit() method not found".into());
        }

        let editor = jni.call_object_method(prefs, edit_method, &[]);
        jni.delete_local_ref(prefs);
        if editor.is_null() {
            return Err("edit() returned null".into());
        }

        f(&jni, editor);

        // Commit
        let editor_cls = jni.get_object_class(editor);
        let commit = jni.get_method_id(editor_cls, b"commit\0", b"()Z\0");
        jni.delete_local_ref(editor_cls);
        if !commit.is_null() {
            jni.call_boolean_method(editor, commit, &[]);
        }
        jni.delete_local_ref(editor);
    }
    Ok(())
}
