//! Instagram-style vertical scrollable feed.
//!
//! Photo cards with user avatars, like/comment/share buttons, captions,
//! and like counts. Tap the heart to toggle likes.

use gpui::{div, img, prelude::*, px, rgb};

use super::{Router, LIGHT_CARD_BG, LIGHT_TEXT, RED, SURFACE0, SURFACE1, TEXT, SUBTEXT, LIGHT_SUBTEXT};

/// Feed post data.
struct FeedPost {
    username: &'static str,
    avatar_color: u32,
    image_color: u32,
    caption: &'static str,
    likes: u32,
    comments: u32,
    time_ago: &'static str,
    /// Picsum photo ID for the post image.
    photo_id: u32,
}

const POSTS: &[FeedPost] = &[
    FeedPost {
        username: "alex_travels",
        avatar_color: 0xE91E63,
        image_color: 0x1565C0,
        caption: "Sunset over the mountains. Nothing beats this view!",
        likes: 1243,
        comments: 89,
        time_ago: "2h",
        photo_id: 29,
    },
    FeedPost {
        username: "foodie_jordan",
        avatar_color: 0xFF9800,
        image_color: 0x2E7D32,
        caption: "Homemade pasta from scratch. Recipe coming soon!",
        likes: 892,
        comments: 156,
        time_ago: "4h",
        photo_id: 292,
    },
    FeedPost {
        username: "morgan.designs",
        avatar_color: 0x9C27B0,
        image_color: 0x6A1B9A,
        caption: "New UI concept I've been working on. Thoughts?",
        likes: 2105,
        comments: 243,
        time_ago: "6h",
        photo_id: 180,
    },
    FeedPost {
        username: "riley_music",
        avatar_color: 0x00BCD4,
        image_color: 0xBF360C,
        caption: "Studio session vibes. New track drops Friday!",
        likes: 567,
        comments: 45,
        time_ago: "8h",
        photo_id: 453,
    },
    FeedPost {
        username: "casey_codes",
        avatar_color: 0x4CAF50,
        image_color: 0x37474F,
        caption: "Finally shipped the feature. Time for coffee.",
        likes: 1890,
        comments: 167,
        time_ago: "12h",
        photo_id: 1060,
    },
    FeedPost {
        username: "quinn_surf",
        avatar_color: 0x03A9F4,
        image_color: 0x0097A7,
        caption: "Perfect barrel today. The ocean was on fire!",
        likes: 3456,
        comments: 312,
        time_ago: "1d",
        photo_id: 1053,
    },
];

pub fn render(router: &mut Router, cx: &mut gpui::Context<Router>) -> impl IntoElement {
    let dark = router.dark_mode;
    let text_color = if dark { TEXT } else { LIGHT_TEXT };
    let sub_text = if dark { SUBTEXT } else { LIGHT_SUBTEXT };
    let _card_bg = if dark { SURFACE0 } else { LIGHT_CARD_BG };
    let divider = if dark { SURFACE1 } else { 0xDADAE0 };

    let mut feed = div().flex().flex_col().w_full();

    for (i, post) in POSTS.iter().enumerate() {
        let liked = router.feed_likes[i];
        let like_count = post.likes + if liked { 1 } else { 0 };

        feed = feed.child(
            div()
                .flex()
                .flex_col()
                .w_full()
                // Header row: avatar + username + time
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .px_3()
                        .py_2()
                        .child(
                            // Avatar circle
                            div()
                                .size(px(36.0))
                                .rounded_full()
                                .bg(rgb(post.avatar_color))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(0xFFFFFF))
                                        .child(post.username.chars().next().unwrap_or('?').to_uppercase().to_string()),
                                ),
                        )
                        .child(
                            div()
                                .flex_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_color))
                                        .child(post.username.to_string()),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(sub_text))
                                .child(post.time_ago.to_string()),
                        ),
                )
                // Image area — picsum.photos with colored fallback
                .child({
                    let photo_url: gpui::SharedString = format!(
                        "https://picsum.photos/id/{}/800/640",
                        post.photo_id
                    ).into();
                    div()
                        .w_full()
                        .h(px(320.0))
                        .bg(rgb(post.image_color))
                        .child(
                            img(photo_url)
                                .w_full()
                                .h(px(320.0))
                                .object_fit(gpui::ObjectFit::Cover)
                                .id(format!("feed-img-{}", i)),
                        )
                })
                // Action buttons row
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_4()
                        .px_3()
                        .py_2()
                        .child({
                            let idx = i;
                            div()
                                .text_xl()
                                .text_color(rgb(if liked { RED } else { text_color }))
                                .child(if liked { "♥" } else { "♡" })
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.feed_likes[idx] = !this.feed_likes[idx];
                                        cx.notify();
                                    }),
                                )
                        })
                        .child(
                            div()
                                .text_xl()
                                .text_color(rgb(text_color))
                                .child("💬"),
                        )
                        .child(
                            div()
                                .text_xl()
                                .text_color(rgb(text_color))
                                .child("↗"),
                        )
                        // Spacer
                        .child(div().flex_1())
                        .child(
                            div()
                                .text_xl()
                                .text_color(rgb(text_color))
                                .child("🔖"),
                        ),
                )
                // Like count
                .child(
                    div()
                        .px_3()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(text_color))
                                .child(format!("{} likes", format_count(like_count))),
                        ),
                )
                // Caption
                .child(
                    div()
                        .px_3()
                        .pb_2()
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .gap_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(text_color))
                                        .child(post.username.to_string()),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(sub_text))
                                        .child(post.caption.to_string()),
                                ),
                        ),
                )
                // Comments link
                .child(
                    div()
                        .px_3()
                        .pb_3()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(sub_text))
                                .child(format!("View all {} comments", post.comments)),
                        ),
                )
                // Divider
                .child(div().w_full().h(px(1.0)).bg(rgb(divider))),
        );
    }

    feed
}

fn format_count(n: u32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
