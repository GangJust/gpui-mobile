//! Tinder-style swipeable card stack.
//!
//! Drag cards left (nope) or right (like). Cards are stacked with a slight
//! offset. The top card follows the finger and rotates proportionally to
//! the horizontal drag distance.

use gpui::{div, prelude::*, px, rgb};

use super::{Router, BLUE, GREEN, LIGHT_CARD_BG, LIGHT_TEXT, RED, SURFACE0, TEXT, YELLOW};

/// Demo profile cards.
const PROFILES: &[Profile] = &[
    Profile { name: "Alex", age: 28, bio: "Coffee enthusiast. Hiking on weekends.", color: 0xE91E63 },
    Profile { name: "Jordan", age: 25, bio: "Photographer & world traveler.", color: 0x9C27B0 },
    Profile { name: "Casey", age: 31, bio: "Software engineer. Cat person.", color: 0x3F51B5 },
    Profile { name: "Morgan", age: 27, bio: "Yoga instructor. Plant parent.", color: 0x009688 },
    Profile { name: "Riley", age: 24, bio: "Music producer. Night owl.", color: 0xFF9800 },
    Profile { name: "Taylor", age: 29, bio: "Chef by day, gamer by night.", color: 0x795548 },
    Profile { name: "Quinn", age: 26, bio: "Surfer. Beach lover. Dog dad.", color: 0x00BCD4 },
    Profile { name: "Avery", age: 30, bio: "Startup founder. Marathon runner.", color: 0x4CAF50 },
];

struct Profile {
    name: &'static str,
    age: u32,
    bio: &'static str,
    color: u32,
}

pub fn render(router: &mut Router, cx: &mut gpui::Context<Router>) -> impl IntoElement {
    let dark = router.dark_mode;
    let text_color = if dark { TEXT } else { LIGHT_TEXT };
    let _card_bg = if dark { SURFACE0 } else { LIGHT_CARD_BG };
    let idx = router.swiper_index;
    let drag_x = router.swiper_drag_x;
    let all_swiped = idx >= PROFILES.len();

    let mut root = div()
        .flex()
        .flex_col()
        .flex_1()
        .items_center()
        .gap_4()
        .px_4()
        .py_4();

    if all_swiped {
        // All cards swiped — show reset
        root = root
            .child(div().h(px(100.0)))
            .child(
                div()
                    .text_xl()
                    .text_color(rgb(text_color))
                    .child("No more profiles!"),
            )
            .child(div().h(px(20.0)))
            .child(
                div()
                    .px_6()
                    .py_3()
                    .rounded_xl()
                    .bg(rgb(BLUE))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xFFFFFF))
                            .child("Start Over"),
                    )
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.swiper_index = 0;
                            cx.notify();
                        }),
                    ),
            );
        return root;
    }

    // Card stack — show up to 3 cards (back to front)
    let stack_end = (idx + 3).min(PROFILES.len());
    let visible = &PROFILES[idx..stack_end];

    let mut stack = div()
        .w(px(320.0))
        .h(px(420.0))
        .relative();

    for (i, profile) in visible.iter().enumerate().rev() {
        let is_top = i == 0;
        let offset_y = (i as f32) * 8.0;
        let scale_factor = 1.0 - (i as f32) * 0.04;

        let (card_offset_x, _rotation_deg) = if is_top {
            (drag_x, drag_x * 0.08)
        } else {
            (0.0, 0.0)
        };

        // Overlay label based on drag direction
        let label_element = if is_top && drag_x.abs() > 30.0 {
            let (label, label_color) = if drag_x > 0.0 {
                ("LIKE", GREEN)
            } else {
                ("NOPE", RED)
            };
            let opacity = (drag_x.abs() / 120.0).min(1.0);
            Some(
                div()
                    .absolute()
                    .top(px(20.0))
                    .when(drag_x > 0.0, |d| d.left(px(20.0)))
                    .when(drag_x <= 0.0, |d| d.right(px(20.0)))
                    .px_4()
                    .py_2()
                    .rounded_lg()
                    .border_3()
                    .border_color(rgb(label_color))
                    .opacity(opacity)
                    .child(
                        div()
                            .text_xl()
                            .text_color(rgb(label_color))
                            .child(label),
                    ),
            )
        } else {
            None
        };

        let card = div()
            .absolute()
            .top(px(offset_y))
            .left(px(card_offset_x + (1.0 - scale_factor) * 160.0))
            .w(px(320.0 * scale_factor))
            .h(px(420.0 * scale_factor))
            .rounded_3xl()
            .overflow_hidden()
            .bg(rgb(profile.color))
            .flex()
            .flex_col()
            .justify_end()
            // Profile info overlay at bottom
            .child(
                div()
                    .w_full()
                    .px_5()
                    .py_4()
                    .bg(gpui::rgba(0x00000088))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_end()
                            .gap_2()
                            .child(
                                div()
                                    .text_xl()
                                    .text_color(rgb(0xFFFFFF))
                                    .child(profile.name.to_string()),
                            )
                            .child(
                                div()
                                    .text_lg()
                                    .text_color(rgb(0xCCCCCC))
                                    .child(format!("{}", profile.age)),
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xDDDDDD))
                            .mt_1()
                            .child(profile.bio.to_string()),
                    ),
            )
            .children(label_element);

        stack = stack.child(card);
    }

    // Wrap stack in a drag area
    let drag_area = div()
        .w(px(320.0))
        .h(px(420.0))
        .child(stack)
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(|this, _event: &gpui::MouseDownEvent, _window, cx| {
                this.swiper_dragging = true;
                this.swiper_drag_x = 0.0;
                cx.notify();
            }),
        )
        .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _window, cx| {
            if this.swiper_dragging {
                this.swiper_drag_x += event.position.x.as_f32()
                    - event.position.x.as_f32()
                    + (if event.position.x.as_f32() > 0.0 {
                        event.position.x.as_f32() * 0.0 // need delta
                    } else {
                        0.0
                    });
                // Use raw position offset from center of card
                let center_x = 160.0;
                this.swiper_drag_x = event.position.x.as_f32() - center_x;
                cx.notify();
            }
        }))
        .on_mouse_up(
            gpui::MouseButton::Left,
            cx.listener(|this, _event: &gpui::MouseUpEvent, _window, cx| {
                if this.swiper_dragging {
                    this.swiper_dragging = false;
                    // Swipe threshold
                    if this.swiper_drag_x > 100.0 || this.swiper_drag_x < -100.0 {
                        let direction = if this.swiper_drag_x > 0.0 { "LIKED" } else { "NOPED" };
                        if this.swiper_index < PROFILES.len() {
                            log::info!("Swiper: {} {}", direction, PROFILES[this.swiper_index].name);
                            this.swiper_index += 1;
                        }
                    }
                    this.swiper_drag_x = 0.0;
                    cx.notify();
                }
            }),
        );

    root = root.child(drag_area);

    // Action buttons row
    root = root.child(
        div()
            .flex()
            .flex_row()
            .gap_6()
            .mt_4()
            .child(action_btn("X", RED, cx.listener(|this, _, _, cx| {
                if this.swiper_index < PROFILES.len() {
                    log::info!("Swiper: NOPED {}", PROFILES[this.swiper_index].name);
                    this.swiper_index += 1;
                    this.swiper_drag_x = 0.0;
                }
                cx.notify();
            })))
            .child(action_btn("*", YELLOW, cx.listener(|_this, _, _, cx| {
                log::info!("Swiper: SUPERLIKED");
                cx.notify();
            })))
            .child(action_btn("~", GREEN, cx.listener(|this, _, _, cx| {
                if this.swiper_index < PROFILES.len() {
                    log::info!("Swiper: LIKED {}", PROFILES[this.swiper_index].name);
                    this.swiper_index += 1;
                    this.swiper_drag_x = 0.0;
                }
                cx.notify();
            }))),
    );

    root
}

fn action_btn(
    icon: &str,
    color: u32,
    handler: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(56.0))
        .rounded_full()
        .border_2()
        .border_color(rgb(color))
        .child(
            div()
                .text_xl()
                .text_color(rgb(color))
                .child(icon.to_string()),
        )
        .on_mouse_down(gpui::MouseButton::Left, handler)
}
