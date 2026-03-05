//! In-app browser screen with back, reload, stop, and close controls.
//!
//! The WebView is a native platform overlay — GPUI renders the toolbar and
//! the native WKWebView/android.webkit.WebView sits on top of the rest.

use gpui::{div, prelude::*, rgb};

use super::{Router, BLUE, GREEN, LIGHT_CARD_BG, LIGHT_TEXT, RED, SURFACE0, TEXT, YELLOW};

/// Render the in-app browser screen.
///
/// Layout: URL bar at top, action buttons row, then the rest of the screen
/// is occupied by the native WebView overlay (not managed by GPUI).
pub fn render(router: &Router, cx: &mut gpui::Context<Router>) -> impl IntoElement {
    let dark = router.dark_mode;
    let text_color = if dark { TEXT } else { LIGHT_TEXT };
    let card_bg = if dark { SURFACE0 } else { LIGHT_CARD_BG };
    let has_webview = router.webview_handle.is_some();
    let url_display = router.webview_url.clone();

    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap_2()
        .px_3()
        .py_3()
        // URL display
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .rounded_xl()
                .bg(rgb(card_bg))
                .px_4()
                .py_3()
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(rgb(text_color))
                        .child(url_display),
                ),
        )
        // Action buttons row
        .child(
            div()
                .flex()
                .flex_row()
                .gap_2()
                // Back button
                .child(browser_btn(
                    "Back",
                    BLUE,
                    has_webview,
                    cx.listener(|this, _, _, cx| {
                        if let Some(ptr) = this.webview_handle {
                            let handle = gpui_mobile::packages::webview::WebViewHandle { ptr };
                            let _ = gpui_mobile::packages::webview::go_back(&handle);
                            std::mem::forget(handle);
                        }
                        cx.notify();
                    }),
                ))
                // Reload button
                .child(browser_btn(
                    "Reload",
                    GREEN,
                    has_webview,
                    cx.listener(|this, _, _, cx| {
                        if let Some(ptr) = this.webview_handle {
                            let handle = gpui_mobile::packages::webview::WebViewHandle { ptr };
                            let _ = gpui_mobile::packages::webview::reload(&handle);
                            std::mem::forget(handle);
                        }
                        cx.notify();
                    }),
                ))
                // Stop button
                .child(browser_btn(
                    "Stop",
                    YELLOW,
                    has_webview,
                    cx.listener(|this, _, _, cx| {
                        if let Some(ptr) = this.webview_handle {
                            let handle = gpui_mobile::packages::webview::WebViewHandle { ptr };
                            let _ = gpui_mobile::packages::webview::stop_loading(&handle);
                            std::mem::forget(handle);
                        }
                        cx.notify();
                    }),
                ))
                // Close button
                .child(browser_btn(
                    "Close",
                    RED,
                    has_webview,
                    cx.listener(|this, _, _, cx| {
                        if let Some(ptr) = this.webview_handle.take() {
                            let handle = gpui_mobile::packages::webview::WebViewHandle { ptr };
                            let _ = gpui_mobile::packages::webview::dismiss(handle);
                        }
                        cx.notify();
                    }),
                )),
        )
        // Open / quick-link buttons
        .child(
            div()
                .flex()
                .flex_row()
                .gap_2()
                .child(open_url_btn(
                    "Google",
                    "https://google.com",
                    BLUE,
                    cx,
                ))
                .child(open_url_btn(
                    "GitHub",
                    "https://github.com",
                    0x333333,
                    cx,
                ))
                .child(open_url_btn(
                    "Zed.dev",
                    "https://zed.dev",
                    GREEN,
                    cx,
                )),
        )
        // Load HTML demo
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .px_4()
                .py_3()
                .rounded_xl()
                .bg(rgb(0xFA7B17))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xFFFFFF))
                        .child("Load Custom HTML"),
                )
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        // Dismiss existing
                        if let Some(ptr) = this.webview_handle.take() {
                            let h = gpui_mobile::packages::webview::WebViewHandle { ptr };
                            let _ = gpui_mobile::packages::webview::dismiss(h);
                        }
                        let settings = gpui_mobile::packages::webview::WebViewSettings::default();
                        let html = r#"<html><body style="background:#1e1f25;color:#e2e2e9;display:flex;align-items:center;justify-content:center;height:100vh;font-family:system-ui;flex-direction:column"><h1>GPUI WebView</h1><p>Custom HTML loaded successfully</p><button onclick="document.body.style.background='#4285F4'" style="padding:12px 24px;font-size:16px;border:none;border-radius:8px;background:#34A853;color:white;margin-top:16px">Change Color</button></body></html>"#;
                        match gpui_mobile::packages::webview::load_html(html, &settings) {
                            Ok(handle) => {
                                this.webview_url = "about:blank".into();
                                this.webview_handle = Some(handle.ptr);
                                std::mem::forget(handle);
                                log::info!("WebView: loaded custom HTML");
                            }
                            Err(e) => log::error!("WebView HTML error: {e}"),
                        }
                        cx.notify();
                    }),
                ),
        )
        // Status
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .py_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(if has_webview { GREEN } else { 0x666666 }))
                        .child(if has_webview {
                            "WebView is active — use Close to dismiss"
                        } else {
                            "No active WebView — tap a link above to open"
                        }),
                ),
        )
}

fn browser_btn(
    label: &str,
    color: u32,
    enabled: bool,
    handler: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    let opacity = if enabled { 1.0 } else { 0.4 };
    div()
        .flex_1()
        .flex()
        .items_center()
        .justify_center()
        .px_2()
        .py_3()
        .rounded_xl()
        .bg(rgb(color))
        .opacity(opacity)
        .child(
            div()
                .text_xs()
                .text_color(rgb(0xFFFFFF))
                .child(label.to_string()),
        )
        .when(enabled, |d| {
            d.on_mouse_down(gpui::MouseButton::Left, handler)
        })
}

fn open_url_btn(
    label: &str,
    url: &'static str,
    color: u32,
    cx: &mut gpui::Context<Router>,
) -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .items_center()
        .justify_center()
        .px_2()
        .py_3()
        .rounded_xl()
        .bg(rgb(color))
        .child(
            div()
                .text_sm()
                .text_color(rgb(0xFFFFFF))
                .child(label.to_string()),
        )
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(move |this, _, _, cx| {
                // Dismiss existing
                if let Some(ptr) = this.webview_handle.take() {
                    let h = gpui_mobile::packages::webview::WebViewHandle { ptr };
                    let _ = gpui_mobile::packages::webview::dismiss(h);
                }
                let settings = gpui_mobile::packages::webview::WebViewSettings::default();
                match gpui_mobile::packages::webview::load_url(url, &settings) {
                    Ok(handle) => {
                        this.webview_url = url.into();
                        this.webview_handle = Some(handle.ptr);
                        std::mem::forget(handle);
                        log::info!("WebView: loaded {url}");
                    }
                    Err(e) => log::error!("WebView URL error: {e}"),
                }
                cx.notify();
            }),
        )
}
