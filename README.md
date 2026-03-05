# GPUI Mobile

A high-performance mobile platform layer for [GPUI](https://github.com/zed-industries/zed), enabling Rust-based UI applications to run natively on **iOS** and **Android**.

[![Build & Test](https://github.com/itsbalamurali/gpui-mobile/actions/workflows/ci.yml/badge.svg)](https://github.com/itsbalamurali/gpui-mobile/actions/workflows/ci.yml)

## Overview

`gpui-mobile` implements the **real `gpui::Platform` trait** from the [Zed](https://github.com/zed-industries/zed) editor for mobile targets. It follows the same architecture as [`gpui_linux`](https://github.com/zed-industries/zed/tree/main/crates/gpui_linux) — a separate crate that implements `gpui::Platform` for a specific OS family.

| Platform | Renderer | Windowing | Text | Dispatcher |
|----------|----------|-----------|------|------------|
| **iOS** | Metal via [wgpu](https://wgpu.rs/) | UIKit (`UIWindow` + `CAMetalLayer`) | CoreText | Grand Central Dispatch |
| **Android** | Vulkan / GL via [wgpu](https://wgpu.rs/) | NDK (`ANativeWindow`) | [cosmic-text](https://github.com/pop-os/cosmic-text) + [swash](https://github.com/dfrg/swash) | `ALooper` + thread pool |

### Key Dependencies

| Crate | Source | Used For |
|-------|--------|----------|
| `gpui` | [zed-industries/zed](https://github.com/zed-industries/zed) | `Platform` trait, event types, geometry, text system traits |
| `gpui_wgpu` | [zed-industries/zed](https://github.com/zed-industries/zed) | wgpu renderer, `CosmicTextSystem`, `WgpuAtlas` |
| `wgpu` 28.x | crates.io | Metal backend (iOS), Vulkan/GL backend (Android) |
| `cosmic-text` 0.17 | crates.io | Text shaping & layout (Android) |
| `core-text` 21 | crates.io | Text shaping (iOS, via CoreText framework) |

## Features

- **Full `gpui::Platform` implementation** for iOS and Android
- **GPU-accelerated rendering** via Metal (iOS) and Vulkan/GL (Android)
- **Touch input** with tap-vs-scroll state machine
- **Momentum scrolling** (inertia / fling) — smooth, decelerating scroll after finger lift, matching native platform feel
- **Emoji rendering** — bundled CBDT bitmap emoji font for Android 13+ (where system font uses COLR v1 unsupported by swash)
- **Keyboard input** — hardware and software keyboard support with full keycode mapping
- **Safe area insets** — notch / home indicator / status bar awareness
- **Dark mode** — responds to system appearance changes
- **Example app** with multiple screens: Home, Counter, About, Settings, Components (Apple Glass + Material Design), Animations, Shaders

## Project Structure

```
gpui/
├── Cargo.toml                  # Crate manifest (gpui-mobile)
├── src/
│   ├── lib.rs                  # Crate root — re-exports, platform dispatch
│   ├── momentum.rs             # Shared momentum/inertia scrolling engine
│   ├── ios/                    # iOS platform implementation
│   │   ├── mod.rs              # Module root + geometry helpers
│   │   ├── platform.rs         # IosPlatform (impl gpui::Platform)
│   │   ├── window.rs           # IosWindow (UIWindow + CAMetalLayer + wgpu)
│   │   ├── display.rs          # IosDisplay (UIScreen wrapper)
│   │   ├── dispatcher.rs       # IosDispatcher (GCD)
│   │   ├── events.rs           # UITouch → gpui::PlatformInput
│   │   ├── ffi.rs              # C-ABI bridge for ObjC app delegates
│   │   ├── text_input.rs       # HID key-code → gpui::Keystroke
│   │   └── text_system.rs      # CoreText text shaping
│   └── android/                # Android platform implementation
│       ├── mod.rs              # Module root
│       ├── platform.rs         # AndroidPlatform (impl gpui::Platform)
│       ├── window.rs           # AndroidWindow (ANativeWindow + wgpu)
│       ├── renderer.rs         # wgpu device/queue/swapchain
│       ├── display.rs          # AndroidDisplay (AConfiguration density)
│       ├── dispatcher.rs       # AndroidDispatcher (ALooper)
│       ├── keyboard.rs         # NDK keycodes → gpui::Keystroke
│       ├── atlas.rs            # GPU texture atlas (etagere)
│       ├── text.rs             # AndroidTextSystem (cosmic-text + swash)
│       └── jni_entry.rs        # NativeActivity lifecycle + event loop
├── example/
│   ├── Cargo.toml              # Example app crate
│   ├── build.sh                # Unified build & run script
│   ├── src/
│   │   ├── lib.rs              # App entry points (iOS + Android)
│   │   ├── main.rs             # Desktop stub
│   │   ├── screens/            # UI screens (Home, Counter, About, etc.)
│   │   └── demos/              # Interactive demos (Animations, Shaders)
│   ├── ios/                    # Xcode project (XcodeGen)
│   │   ├── project.yml
│   │   ├── Sources/            # Swift AppDelegate + bridging
│   │   └── Resources/          # Assets, icons
│   └── android/
│       └── gradle/             # Gradle project
│           ├── build.gradle.kts
│           └── app/
│               ├── build.gradle.kts
│               ├── src/main/
│               │   ├── AndroidManifest.xml
│               │   ├── assets/fonts/  # Bundled CBDT NotoColorEmoji
│               │   └── res/           # Icons, strings
│               └── ...
└── .github/
    └── workflows/
        └── ci.yml              # CI: build + test for iOS & Android
```

## Quick Start

### Prerequisites

- **Rust** 1.75+
- **iOS**: macOS with Xcode 15+, [XcodeGen](https://github.com/yonaskolb/XcodeGen) (`brew install xcodegen`)
- **Android**: Android SDK + NDK r25+, [cargo-ndk](https://github.com/nickelc/cargo-ndk) (`cargo install cargo-ndk`)

### Build & Run

The unified build script handles everything:

```bash
cd example

# iOS — build, sign, install & launch on connected iPhone
./build.sh ios --device

# iOS — simulator
./build.sh ios --simulator

# Android — build, package APK, install & launch on connected device
./build.sh android --device

# Android — emulator
./build.sh android --emulator

# Release builds
./build.sh ios --device --release
./build.sh android --device --release
```

### Manual Build Steps

#### iOS

```bash
# Add target
rustup target add aarch64-apple-ios

# Build the example (builds gpui-mobile as dependency)
cd example
cargo build --target aarch64-apple-ios --features "ios,font-kit"

# Generate Xcode project & build
cd ios
xcodegen generate --spec project.yml
xcodebuild -project GpuiExample.xcodeproj -scheme GpuiExample \
  -destination "generic/platform=iOS" build
```

#### Android

```bash
# Add target
rustup target add aarch64-linux-android

# Build the shared library
cd example
cargo ndk -t arm64-v8a -o android/gradle/app/src/main/jniLibs build

# Assemble & install APK
cd android/gradle
./gradlew assembleDebug
adb install -r app/build/outputs/apk/debug/app-debug.apk

# Launch
adb shell am start -n "dev.gpui.mobile.example/android.app.NativeActivity"
```

### Host (documentation / CI check)

```bash
# Compiles the non-mobile fallback (pulls gpui + gpui_wgpu from Zed repo)
cargo check

# Run unit tests (momentum scrolling, keyboard mapping, etc.)
cargo test
```

## Architecture

### iOS

```
IosPlatform (impl gpui::Platform)
  ├── IosDispatcher         — GCD main queue + background queue
  ├── IosWindow             — UIWindow + CAMetalLayer + wgpu Metal renderer
  │     ├── touch handler   — tap/scroll state machine + velocity tracker
  │     └── momentum pump   — driven by CADisplayLink via FFI
  ├── IosDisplay            — UIScreen bounds + scale factor
  ├── events                — UITouch → ScrollWheel / MouseDown / MouseMove / MouseUp
  └── text_system           — CoreText font shaping + rendering
```

### Android

```
AndroidPlatform (impl gpui::Platform)
  ├── AndroidDispatcher     — ALooper (foreground) + thread pool (background)
  ├── AndroidWindow         — ANativeWindow + wgpu Vulkan/GL renderer
  │     ├── touch handler   — tap/scroll state machine + velocity tracker
  │     └── momentum pump   — driven by event loop frame callback
  ├── AndroidDisplay        — AConfiguration density + window geometry
  ├── AndroidTextSystem     — cosmic-text shaping + swash rasterisation
  │     └── emoji           — CBDT bitmap fallback for COLR v1 system fonts
  └── jni_entry             — NativeActivity lifecycle + input dispatch
```

### Momentum Scrolling

Both platforms share a common momentum engine (`src/momentum.rs`):

- **VelocityTracker** — ring buffer of recent touch samples with weighted least-squares velocity estimation
- **MomentumScroller** — exponential-decay animation producing decelerating scroll deltas each frame
- On finger lift: compute release velocity → start fling
- On new touch: instantly cancel active fling
- Tuned to match native iOS `UIScrollView` / Android `OverScroller` feel

### Emoji Rendering (Android)

Android 13+ ships `NotoColorEmoji.ttf` using COLR v1 color outlines, which `swash` cannot render. The platform detects this at startup by checking for CBDT table presence, and falls back to a bundled CBDT-based NotoColorEmoji (v2.042) from APK assets. Emoji glyphs are rendered using `Format::CustomSubpixel` for full-color RGBA output.

## Platform Support

| Platform | Status | Min Version | GPU Backend |
|----------|--------|-------------|-------------|
| iOS (device) | ✅ | iOS 13.0+ | Metal |
| iOS (simulator) | ✅ | iOS 13.0+ | Metal (simulated) |
| Android (arm64) | ✅ | API 26+ | Vulkan (preferred), GL ES 3.0 (fallback) |
| Android (armv7) | ⚠️ Untested | API 26+ | Vulkan / GL ES |
| Android (x86_64) | ⚠️ Emulator | API 26+ | Vulkan / GL ES |
| Host (macOS/Linux) | 🔧 Check only | — | — |

## Cargo Features

| Feature | Description |
|---------|-------------|
| `font-kit` | Enables `font-kit` based font matching on iOS (CoreText) |
| `ios` | (marker) iOS-specific code paths |
| `android` | (marker) Android-specific code paths |

## Example App

The example app demonstrates the full GPUI mobile stack:

| Screen | Description |
|--------|-------------|
| **Home** | Navigation hub with cards linking to all screens |
| **Counter** | Simple interactive counter (tap to increment) |
| **About** | Scrollable tech stack info — tests scroll + momentum |
| **Settings** | Theme toggle, preferences UI |
| **Components** | Apple Glass + Material Design component showcase |
| **Animations** | Physics playground — drag-to-throw, spring animations |
| **Shaders** | WGSL shader gallery with touch-reactive effects |

## CI / GitHub Actions

The repository includes a CI workflow (`.github/workflows/ci.yml`) that:

1. **Checks** the host build (`cargo check`)
2. **Tests** unit tests (`cargo test` — momentum, keyboard, etc.)
3. **Cross-compiles** for iOS (`aarch64-apple-ios`)
4. **Cross-compiles** for Android (`aarch64-linux-android`) using the NDK
5. **Lints** with `cargo clippy`

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Ensure all targets compile:
   ```bash
   cargo check
   cargo check --target aarch64-apple-ios
   cargo check --target aarch64-linux-android  # requires NDK in PATH
   ```
4. Run tests: `cargo test`
5. Run `cargo fmt --all`
6. Open a Pull Request

## License

This project is licensed under any one of the following licenses, at your option:

- [GNU General Public License v3.0 or later](LICENSE-GPL)
- [GNU Affero General Public License v3.0 or later](LICENSE-AGPL)
- [Apache License, Version 2.0](LICENSE-APACHE)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you shall be licensed under the same terms,
without any additional terms or conditions.

## Acknowledgements

- [Zed Industries](https://github.com/zed-industries/zed) — for the GPUI framework
- [wgpu](https://wgpu.rs/) — cross-platform GPU abstraction
- [cosmic-text](https://github.com/pop-os/cosmic-text) — Unicode text shaping
- [swash](https://github.com/dfrg/swash) — font rasterisation
- [Google Noto Fonts](https://github.com/googlefonts/noto-emoji) — bundled emoji font (Apache 2.0)