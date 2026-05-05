# rust-doodle

A sandbox repo for Rust experiments and learning.

## Projects

| Project | Description |
|---------|-------------|
| `cross-platform/hello-rust` | Simple hello world |
| `cross-platform/hello-bevy` | Bevy game engine hello world with custom font and centered text |
| `cross-platform/hello-bevy-advanced` | Bevy 2D demo (text wave, particles, spotlight shader); buildable for Android via cargo-apk |

## Desktop builds

Each crate is independent — no Cargo workspace. Build/run with `--manifest-path`:

```powershell
cargo build --manifest-path cross-platform/<project>/Cargo.toml
cargo run   --manifest-path cross-platform/<project>/Cargo.toml
```

## Android build (`hello-bevy-advanced`)

Targets `aarch64-linux-android` via [`cargo-apk`](https://github.com/rust-mobile/cargo-apk). Only `hello-bevy-advanced` is currently set up for Android; other crates are desktop-only.

For the conceptual deep-dive — activity-backend gotcha, Bevy feature-list workaround, physical-vs-logical pixel handling, diagnostic toolkit, reference Cargo.toml — see [`docs/bevy-android.md`](docs/bevy-android.md). The summary below is just enough to build and deploy.

### One-time toolchain setup (Windows)

- **JDK 17** active for Gradle compatibility (point `JAVA_HOME` at it; ensure `java -version` reports 17, not 21+).
- **Android SDK** + `platform-tools` (provides `adb`); set `ANDROID_HOME`.
- **Android NDK r26+** (install via the Android Studio SDK Manager); set `ANDROID_NDK_HOME`.
- `rustup target add aarch64-linux-android`
- `cargo install cargo-apk`

On the device side, enable **USB debugging** in *Developer options* and accept the RSA fingerprint prompt the first time you connect.

### Build & deploy

```powershell
cd cross-platform/hello-bevy-advanced

adb devices                  # confirm the device is listed and authorized
cargo apk run --lib          # build, install, launch (debug)
cargo apk run --lib --release   # smaller APK, optimized
```

The `--lib` flag is mandatory because the crate also has a `[[bin]]` (the desktop entry); without it cargo-apk doesn't know which target to package.

### Android-specific decisions in this crate

- **Entry point**: `src/lib.rs` hosts `#[bevy_main] pub fn main()`. On Android the macro generates the `android_main` JNI entry; on desktop it's a passthrough. `src/main.rs` is a thin wrapper so `cargo run` keeps working.
- **Activity backend**: NativeActivity. Bevy's default `android-game-activity` is incompatible with cargo-apk's APK builder (no Java/dex bundling). Cargo.toml opts out of Bevy defaults and re-enables the equivalent feature set with `android-native-activity` instead.
- **Config loading**: `config.ron` is baked in via `include_str!` on Android (`src/config.rs` is cfg-split). Desktop still reads from disk for fast iteration; editing the file requires a rebuild on Android.
- **Assets**: cargo-apk bundles `assets/` into the APK; Bevy's default `AAssetManager`-backed loader reads them. The desktop `AssetPlugin` filesystem-path override is cfg-gated to non-Android targets.
- **Input**: the spotlight system reads `Touches` first; on desktop it falls back to the mouse cursor (cfg-gated out on Android, where `cursor_position()` returns a stale `Some(Vec2::ZERO)`). Idle position is the viewport center.
- **Coordinate spaces**: shader uniforms involving screen positions are converted to **physical** pixels in the Rust system before upload (multiplied by `window.scale_factor()`), so the WGSL `frag_pos` and the uniform agree on units. Without this, effects land in the top-left quadrant on HiDPI displays.
- **`configChanges`**: the activity declares a comprehensive set so Android doesn't destroy/recreate it on resize/multi-window/density changes (which would invalidate Bevy's wgpu surface).
- **Library/binary names**: the Android shared library is `libhello_bevy_advanced.so`. The desktop binary is renamed to `hello-bevy-advanced-bin` so Windows doesn't try to write the same `.pdb` filename twice (cargo normalizes hyphens to underscores).
