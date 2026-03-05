use super::{WebViewHandle, WebViewSettings};
use crate::android::jni::{self as jni_helpers, JniExt};
use jni::objects::{JObject, JValue};

pub fn load_url(url: &str, settings: &WebViewSettings) -> Result<WebViewHandle, String> {
    let url = url.to_owned();
    let settings = settings.clone();
    jni_helpers::with_env(|env| {
        let activity = jni_helpers::activity(env)?;

        // WebView webview = new WebView(activity);
        let webview = env
            .new_object(
                jni::jni_str!("android/webkit/WebView"),
                jni::jni_sig!("(Landroid/content/Context;)V"),
                &[JValue::Object(&activity)],
            )
            .e()?;

        configure_webview(env, &webview, &settings)?;

        // webview.loadUrl(url)
        let jurl = env.new_string(&url).e()?;
        let _ = env.call_method(
            &webview,
            jni::jni_str!("loadUrl"),
            jni::jni_sig!("(Ljava/lang/String;)V"),
            &[JValue::Object(&jurl)],
        );
        let _ = env.exception_clear();

        add_to_content_view(env, &activity, &webview)?;

        // Store as a global ref so it survives past this JNI call
        let global = env.new_global_ref(&webview).e()?;
        let ptr = global.as_raw() as usize;
        std::mem::forget(global); // prevent drop — will be cleaned up in dismiss()
        Ok(WebViewHandle { ptr })
    })
}

pub fn load_html(html: &str, settings: &WebViewSettings) -> Result<WebViewHandle, String> {
    let html = html.to_owned();
    let settings = settings.clone();
    jni_helpers::with_env(|env| {
        let activity = jni_helpers::activity(env)?;

        let webview = env
            .new_object(
                jni::jni_str!("android/webkit/WebView"),
                jni::jni_sig!("(Landroid/content/Context;)V"),
                &[JValue::Object(&activity)],
            )
            .e()?;

        configure_webview(env, &webview, &settings)?;

        // webview.loadData(html, "text/html", "UTF-8")
        let jhtml = env.new_string(&html).e()?;
        let mime = env.new_string("text/html").e()?;
        let encoding = env.new_string("UTF-8").e()?;
        let _ = env.call_method(
            &webview,
            jni::jni_str!("loadData"),
            jni::jni_sig!("(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V"),
            &[JValue::Object(&jhtml), JValue::Object(&mime), JValue::Object(&encoding)],
        );
        let _ = env.exception_clear();

        add_to_content_view(env, &activity, &webview)?;

        let global = env.new_global_ref(&webview).e()?;
        let ptr = global.as_raw() as usize;
        std::mem::forget(global);
        Ok(WebViewHandle { ptr })
    })
}

pub fn evaluate_javascript(handle: &WebViewHandle, script: &str) -> Result<(), String> {
    let ptr = handle.ptr;
    let script = script.to_owned();
    jni_helpers::with_env(|env| {
        let webview = unsafe { JObject::from_raw(env, ptr as jni::sys::jobject) };

        let jscript = env.new_string(&script).e()?;
        let _ = env.call_method(
            &webview,
            jni::jni_str!("evaluateJavascript"),
            jni::jni_sig!("(Ljava/lang/String;Landroid/webkit/ValueCallback;)V"),
            &[JValue::Object(&jscript), JValue::Object(&JObject::null())],
        );
        let _ = env.exception_clear();
        std::mem::forget(webview); // don't drop the borrowed ref
        Ok(())
    })
}

pub fn dismiss(handle: WebViewHandle) -> Result<(), String> {
    let ptr = handle.ptr;
    jni_helpers::with_env(|env| {
        let webview = unsafe { JObject::from_raw(env, ptr as jni::sys::jobject) };

        // Get parent ViewGroup and remove the webview
        let parent = env
            .call_method(
                &webview,
                jni::jni_str!("getParent"),
                jni::jni_sig!("()Landroid/view/ViewParent;"),
                &[],
            )
            .and_then(|v: jni::objects::JValueOwned| v.l());

        if let Ok(parent) = parent {
            if !parent.is_null() {
                let _ = env.call_method(
                    &parent,
                    jni::jni_str!("removeView"),
                    jni::jni_sig!("(Landroid/view/View;)V"),
                    &[JValue::Object(&webview)],
                );
                let _ = env.exception_clear();
            }
        }

        // webview.destroy()
        let _ = env.call_method(&webview, jni::jni_str!("destroy"), jni::jni_sig!("()V"), &[]);
        let _ = env.exception_clear();

        // Delete the global ref via raw JNI
        unsafe {
            let raw_env = env.get_raw();
            let interface: *const jni::sys::JNINativeInterface_ = *raw_env;
            ((*interface).v1_1.DeleteGlobalRef)(raw_env, ptr as jni::sys::jobject);
        }

        Ok(())
    })
}

fn configure_webview(
    env: &mut jni::Env<'_>,
    webview: &JObject<'_>,
    settings: &WebViewSettings,
) -> Result<(), String> {
    // WebSettings ws = webview.getSettings();
    let ws = env
        .call_method(
            webview,
            jni::jni_str!("getSettings"),
            jni::jni_sig!("()Landroid/webkit/WebSettings;"),
            &[],
        )
        .and_then(|v| v.l())
        .e()?;

    // ws.setJavaScriptEnabled(...)
    let _ = env.call_method(
        &ws,
        jni::jni_str!("setJavaScriptEnabled"),
        jni::jni_sig!("(Z)V"),
        &[JValue::Bool(settings.javascript_enabled)],
    );

    // ws.setDomStorageEnabled(...)
    let _ = env.call_method(
        &ws,
        jni::jni_str!("setDomStorageEnabled"),
        jni::jni_sig!("(Z)V"),
        &[JValue::Bool(settings.dom_storage_enabled)],
    );

    // ws.setSupportZoom(...)
    let _ = env.call_method(
        &ws,
        jni::jni_str!("setSupportZoom"),
        jni::jni_sig!("(Z)V"),
        &[JValue::Bool(settings.zoom_enabled)],
    );

    // User agent
    if let Some(ref ua) = settings.user_agent {
        if let Ok(jua) = env.new_string(ua) {
            let _ = env.call_method(
                &ws,
                jni::jni_str!("setUserAgentString"),
                jni::jni_sig!("(Ljava/lang/String;)V"),
                &[JValue::Object(&jua)],
            );
        }
    }

    let _ = env.exception_clear();
    Ok(())
}

fn add_to_content_view(
    env: &mut jni::Env<'_>,
    activity: &JObject<'_>,
    webview: &JObject<'_>,
) -> Result<(), String> {
    // FrameLayout.LayoutParams params = new FrameLayout.LayoutParams(MATCH_PARENT, MATCH_PARENT)
    let params = env
        .new_object(
            jni::jni_str!("android/widget/FrameLayout$LayoutParams"),
            jni::jni_sig!("(II)V"),
            &[JValue::Int(-1), JValue::Int(-1)], // MATCH_PARENT = -1
        )
        .e()?;

    // activity.addContentView(webview, params)
    let _ = env.call_method(
        activity,
        jni::jni_str!("addContentView"),
        jni::jni_sig!("(Landroid/view/View;Landroid/view/ViewGroup$LayoutParams;)V"),
        &[JValue::Object(webview), JValue::Object(&params)],
    );
    let _ = env.exception_clear();
    Ok(())
}
