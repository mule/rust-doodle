# rust-doodle

A sandbox repo for Rust experiments and learning.

## Projects

| Project | Description |
|---------|-------------|
| `cross-platform/hello-rust` | Simple hello world |
| `cross-platform/hello-bevy` | Bevy game engine hello world with custom font and centered text |
| `cross-platform/hello-bevy-advanced` | Bevy 2D demo (text wave, particles, spotlight shader); buildable for Android via xbuild |

## Desktop builds

Each crate is independent — no Cargo workspace. Build/run with `--manifest-path`:

```powershell
cargo build --manifest-path cross-platform/<project>/Cargo.toml
cargo run   --manifest-path cross-platform/<project>/Cargo.toml
```

## Android build (`hello-bevy-advanced`)

Targets `aarch64-linux-android` via [xbuild](https://github.com/rust-mobile/xbuild). Only `hello-bevy-advanced` is currently set up for Android; other crates are desktop-only.

> **Status:** code structure is prepared and compiles cleanly on desktop. End-to-end Android build/deploy has not yet been verified on a device — see [issue #5](https://github.com/mule/rust-doodle/issues/5) for progress.

### One-time toolchain setup (Windows)

- **JDK 17** active for Gradle compatibility (point `JAVA_HOME` at it; ensure `java -version` reports 17, not 21+).
- **Android SDK** + `platform-tools` (provides `adb`); set `ANDROID_HOME`.
- **Android NDK r26+** (install via the Android Studio SDK Manager); set `ANDROID_NDK_HOME`.
- `rustup target add aarch64-linux-android`
- `cargo install xbuild`
- Verify the environment: `x doctor`

On the device side, enable **USB debugging** in *Developer options* and accept the RSA fingerprint prompt the first time you connect.

### Build & deploy

```powershell
cd cross-platform/hello-bevy-advanced

x doctor                                            # verify environment is green
x devices                                           # list connected Android devices
x build --platform android --arch arm64 --release   # produce an APK
x run --device adb:<device-id>                      # deploy & launch
```

### Android-specific behavior in this crate

- **Entry point**: `src/lib.rs` hosts `#[bevy_main] pub fn main()`. On Android the macro generates the `android_main` JNI entry; on desktop it's a passthrough. `src/main.rs` is a thin wrapper that calls into the library so `cargo run` keeps working.
- **Config loading**: `config.ron` is baked into the binary at compile time via `include_str!` (see the cfg-gated paths in `src/config.rs`). Editing the file requires a rebuild on Android. Desktop still reads from disk for fast iteration.
- **Assets**: Bevy's default `AAssetManager`-backed asset loader is used on Android; the desktop `AssetPlugin` filesystem-path override is cfg-gated to non-Android targets.
- **Input**: the spotlight system reads `Touches` first, falls back to the mouse cursor on desktop, and idles at the viewport center when neither is active.
- **Library/binary names**: the Android shared library is `libhello_bevy_advanced.so`. The desktop binary is renamed to `hello-bevy-advanced-bin` so Windows doesn't try to write the same `.pdb` filename twice (cargo normalizes hyphens to underscores).
