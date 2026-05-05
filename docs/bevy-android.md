# Bevy on Android — a working-developer's guide

Practical lessons from porting a Bevy 0.18 app to a Samsung Galaxy Tab S9 (Snapdragon 8 Gen 2, Android 14, scale_factor ~2.0). Focused on Windows hosts, but most of the conceptual material applies anywhere.

This guide is organised so you can either skim the **Quick start** and follow the cookbook, or read **Concepts** to understand *why* things are the way they are. The "gotchas" you don't catch on day one are usually conceptual, not procedural.

---

## Quick start

For a brand-new Bevy crate you want to run on Android:

1. **Install once:** `cargo install cargo-apk`, `rustup target add aarch64-linux-android`. Have an Android NDK installed (NDK r26+, JDK 17 active for Gradle compatibility).
2. **Cargo.toml:** make the crate a `cdylib` + `rlib` hybrid, opt out of Bevy's default features and re-list everything you need *minus* `android-game-activity`, plus `android-native-activity`. See [§ Reference Cargo.toml](#reference-cargotoml).
3. **`#[bevy_main]`:** put your `App` setup in `src/lib.rs` under a `#[bevy_main] pub fn main()`. Have a tiny `src/main.rs` that just calls into the library — that's your desktop entry.
4. **Asset paths:** cfg-gate the desktop `AssetPlugin` filesystem override so on Android, Bevy uses its default `AAssetManager`-backed loader.
5. **Build & deploy:** `cargo apk run --lib` (the `--lib` is mandatory if your crate also has a `[[bin]]`).
6. **If it crashes on launch**, run `adb logcat -d` and search for `FATAL` and `UnsatisfiedLinkError` — those are the first two questions you need to answer.

That's the happy path. Everything below is the underwater bulk of the iceberg.

---

## Concepts

### Why NativeActivity, not GameActivity

Bevy 0.18's default features enable `android-game-activity` — meaning your `.so` exports the modern GameActivity bootstrap (`Java_androidx_games_GameActivity_*`). GameActivity gives better text input, more reliable lifecycle hooks, and supports newer Android features.

But `cargo-apk` (and `xbuild`) **only ever generate `android.app.NativeActivity` manifests**. This is a design choice — those tools intentionally don't invoke `javac`/`d8`, so they can't bundle the GameActivity Java glue (`classes.dex`) that the runtime needs to instantiate the activity class. Confirmed in `ndk-build-0.10.0/src/manifest.rs:331` (the `default_activity_name` literal).

The runtime mismatch is severe:
- Manifest says NativeActivity → Android calls `ANativeActivity_onCreate` on your `.so`.
- Your `.so` only exports GameActivity entry points.
- App crashes at launch with `java.lang.UnsatisfiedLinkError: undefined symbol: ANativeActivity_onCreate`.

**Two ways out:**

| Option | Tool | What changes |
|--------|------|--------------|
| **A** *(this repo)* | cargo-apk | Switch Bevy to `android-native-activity` feature. Verbose Cargo.toml, but no Java toolchain |
| **B** | Gradle / Android Studio | Keep GameActivity (Bevy default). Maintain a Gradle project alongside your Cargo crate that compiles+dexes the GameActivity Java sources from the `android-activity` crate and bundles your `.so`. More setup but unlocks the full Android ecosystem |

For learning projects and self-contained Bevy apps, **A** is dramatically simpler. Switch to **B** only when you need text input, IMEs, or other GameActivity-specific features.

### The Bevy feature-list workaround

Cargo features are strictly additive — there's no "subtract a sub-feature" syntax. To swap the activity backend you must `default-features = false` and re-list everything Bevy's `default` would have given you, except `android-game-activity`, plus `android-native-activity`.

Two non-obvious traps:

1. **`android-activity` (the crate behind both Bevy backends) enforces mutex with `compile_error!`.** If `game-activity` and `native-activity` are both enabled in the dep graph, your build halts. Verify with:
   ```bash
   cargo tree --target aarch64-linux-android -e features 2>&1 | grep -E "android-(game|native)-activity"
   ```
   You should see *only* `native-activity` in the chain. If `game-activity` shows up too, something else is enabling it transitively.

2. **Bevy 0.18's `ui` feature transitively re-enables `default_platform`**, which drags `android-game-activity` back in:
   ```toml
   ui = [
       "default_app",
       "default_platform",   # ← drags android-game-activity back in
       ...
   ]
   ```
   Same applies to anything else that includes `default_platform` in its expansion. **Use only the leaf features** (`bevy_render`, `2d_api`, `bevy_sprite_render`, etc.) — convenience aliases like `2d`, `3d`, `ui` re-introduce the conflict.

### Coordinate spaces — the silent killer

This bit me hard. The bug looked like "the spotlight effect is stuck in the top-left quadrant" but the root cause is that **Bevy and WGSL disagree on what a pixel is**.

| Coordinate source | Space |
|-------------------|-------|
| `Window::width()` / `height()` (or `resolution.width()`) | **Logical** pixels (divided by `scale_factor`) |
| `Window::physical_width()` / `physical_height()` | **Physical** pixels (true framebuffer size) |
| `Touch::position()` | **Logical** pixels (`bevy_winit::convert_touch_input` takes `LogicalPosition<f64>`) |
| `Window::cursor_position()` | **Logical** pixels |
| WGSL `@builtin(position)` (i.e. `in.position.xy` in a fragment shader) | **Physical** pixels — the GPU doesn't know about scale factors |

On a 1.0-scale-factor desktop these are identical, so the bug never surfaces. On a Tab S9 (~2.0 scale factor) or a HiDPI Mac, mixing them produces effects that land at half the expected position. Top-left quadrant is the classic symptom, because if `viewport_size = logical` (e.g. 800×1280) and you compute `screen_center = viewport_size / 2 = (400, 640)`, but `frag_pos` ranges (0..1600, 0..2560) — the "center" you computed is the **physical top-left quadrant**.

**The fix is consistent units**, picked once and used throughout the data flow. Two reasonable choices:

#### Option 1 — physical pixels everywhere (recommended for shader-heavy effects)

Convert logical → physical in your Rust system before uploading uniforms:

```rust
let scale = window.scale_factor() as f32;
let physical_size = Vec2::new(
    window.physical_width() as f32,
    window.physical_height() as f32,
);

let logical_pos = touches.iter().next().map(|t| t.position());
#[cfg(not(target_os = "android"))]
let logical_pos = logical_pos.or_else(|| window.cursor_position());

uniforms.viewport_size = physical_size;
uniforms.mouse_pos = logical_pos
    .map(|p| p * scale)
    .unwrap_or(physical_size * 0.5);

// If radius/distance constants are configured in logical pixels and you want
// consistent visual size across DPIs, scale them too:
uniforms.radius = config.radius * scale;
```

Then in WGSL, just use `in.position.xy` directly — no further conversion:

```wgsl
let dist = distance(in.position.xy, uniforms.mouse_pos);
```

#### Option 2 — work in logical pixels and let the shader scale

Keep your Rust side in logical units, but multiply by `scale_factor` *inside* the shader. Cleaner if your Rust code passes a logical `viewport_size` to multiple shaders that don't all want to think about DPI.

The win of Option 1: your shader becomes trivially simple. The win of Option 2: your Rust uniforms read like the values you actually wrote in config files (in logical pixels, the same units the rest of Bevy talks about).

This repo uses Option 1; see `cross-platform/hello-bevy-advanced/src/spotlight.rs`.

### Input handling — don't trust `cursor_position()` on Android

`Window::cursor_position()` returns `Some(Vec2::ZERO)` (top-left, viewport-space) on Android, *not* `None`. winit reports a "last known cursor position" inherited from window creation, and on a system with no cursor it never updates.

If your input system has logic like "use touch if available, else fall back to cursor", that fallback path will silently lock to top-left on Android. Cfg-gate the cursor branch:

```rust
let logical_pos = touches.iter().next().map(|t| t.position());
#[cfg(not(target_os = "android"))]
let logical_pos = logical_pos.or_else(|| window.cursor_position());
```

`Touch::position()` is reliable — it's `None` (i.e. `touches.iter().next()` returns `None`) when no fingers are down.

### `configChanges` — surviving multi-window resize

cargo-apk's default Activity declares only `orientation|keyboardHidden|screenSize` as configuration changes the activity handles itself. For each *unset* bit in `configChanges`, Android **destroys and recreates** the activity when that config changes. Bevy's wgpu surface is tied to the original Surface object, so a recreate invalidates the renderer and crashes the app.

On a tablet, you absolutely hit this when:
- The user maximizes/un-maximizes a windowed app
- Multi-window mode resizes the activity
- DPI changes (rare, but possible with external displays)
- Dark/light mode toggle (ui mode)
- Font scale change (accessibility)

Comprehensive set that survives all the above:

```toml
[package.metadata.android.application.activity]
config_changes = "orientation|keyboardHidden|keyboard|screenSize|smallestScreenSize|screenLayout|density|layoutDirection|colorMode|uiMode|locale|fontScale|navigation"
```

Verify the resulting manifest with:
```bash
"D:/AndroidSDK/build-tools/<latest>/aapt2.exe" dump xmltree \
    --file AndroidManifest.xml target/debug/apk/<app>.apk
```
The `configChanges` attribute should show as `0x40007ff4` (or similar large bitmap) instead of the default `0x4a0`.

### Hybrid `cdylib` + `rlib` + `[[bin]]`

Android needs a `.so`. Desktop needs an `.exe`. To share the source, the crate is configured as `crate-type = ["cdylib", "rlib"]` and exposes `pub fn main()` annotated with `#[bevy_main]`:

```rust
// src/lib.rs
#[bevy_main]
pub fn main() {
    // your App::new()...run() here
}
```

```rust
// src/main.rs
fn main() {
    your_crate_name::main();
}
```

`#[bevy_main]` on Android generates a JNI-compatible `android_main` entry; on desktop it's just a passthrough.

**Windows-specific gotcha:** Cargo normalises `-` to `_` in PDB filenames. If your package is `hello-bevy-advanced`, the auto-generated bin name is `hello_bevy_advanced.exe` with `hello_bevy_advanced.pdb`, but the lib also wants to write `hello_bevy_advanced.pdb`. Build emits a "two output filenames will collide" warning. Work around it with an explicit bin name override:

```toml
[[bin]]
name = "hello-bevy-advanced-bin"
path = "src/main.rs"
```

(cargo-apk only consumes the `[lib]` target on Android, so the bin rename only affects desktop builds. On Linux/macOS the warning doesn't fire because debuginfo files use a different naming scheme.)

### Asset loading

Bevy's default Android asset loader uses Android's `AAssetManager`, which reads from the `assets/` directory inside the APK. cargo-apk bundles a project's `assets/` directory automatically when you set `assets = "assets"` in `[package.metadata.android]`.

For desktop, you typically want to point Bevy at your filesystem `assets/` directory. The override only makes sense off-Android:

```rust
let default_plugins = DefaultPlugins.set(WindowPlugin { /* ... */ });

#[cfg(not(target_os = "android"))]
let default_plugins = default_plugins.set(AssetPlugin {
    file_path: "/absolute/path/to/assets".to_string(),
    ..default()
});

App::new().add_plugins(default_plugins)/* ... */;
```

For things you need to read **before** Bevy is up (e.g. parsing a config file to set `ClearColor` before `App::new()`), `AAssetManager` isn't available yet — there's no `AssetServer`. Either embed via `include_str!` on Android:

```rust
#[cfg(not(target_os = "android"))]
pub fn load_config() -> AppConfig {
    let path = format!("{}/config.ron", env!("CARGO_MANIFEST_DIR"));
    let s = std::fs::read_to_string(&path).expect("read config.ron");
    ron::from_str(&s).expect("parse config.ron")
}

#[cfg(target_os = "android")]
pub fn load_config() -> AppConfig {
    const CONFIG_RON: &str = include_str!("../assets/config.ron");
    ron::from_str(CONFIG_RON).expect("parse embedded config.ron")
}
```

…or move the config-dependent setup *into* a Bevy startup system, where `AssetServer` works on both platforms.

---

## Toolchain landscape

This is the build-tool decision matrix as of late 2025/early 2026. Pick once per project, switch only if you outgrow your choice.

| Tool | Android | iOS | Java/dex bundling | Best for |
|------|---------|-----|-------------------|----------|
| **cargo-apk** | ✅ | ❌ | ❌ (NativeActivity only) | Pure-Rust Bevy apps targeting Android only |
| **xbuild** | ✅ | ✅ | ❌ (NativeActivity only) | Cross-platform tooling experiments; iOS-from-non-Mac CI |
| **cargo-mobile2** | ✅ | ✅ | ✅ (generates a real Gradle project) | Production apps wanting GameActivity + iOS in one tool |
| **Gradle / Android Studio** | ✅ | ❌ | ✅ | Apps that need full Android ecosystem (services, content providers, Java/Kotlin code, custom resources) |

**Why cargo-apk for this repo:** simple, well-documented, integrates cleanly with `cargo`. Asset bundling is a one-line metadata key. Deploys with `cargo apk run --lib` over USB. Supports incremental rebuilds — second build is ~25s vs ~14m first time.

**Sigil for switching:** if you find yourself wanting any of these, it's time for cargo-mobile2 or Gradle:
- Text input / IMEs (need GameActivity)
- Custom Android services or content providers
- Multi-activity apps
- iOS device deployment (need either xbuild or cargo-mobile2)
- App store builds (need full release-keystore signing flow that's painful in cargo-apk)

### iOS — looking ahead

Important constraint: **you cannot produce an installable iOS `.ipa` from Windows**, period. Apple's signing toolchain only runs on macOS. From Windows you can cross-compile the Rust artifact (a `.a` static library), but you'll need a Mac (or a cloud-Mac CI like GitHub Actions `macos-latest`) somewhere in the loop.

Tooling-wise: cargo-apk is Android-only. For iOS you'll be looking at xbuild (multi-platform), cargo-mobile2 (multi-platform with Gradle/Xcode generation), or Bevy's own approach for `examples/mobile` which uses raw `cargo build --target aarch64-apple-ios` driven by a hand-written `Makefile` that calls `xcodebuild`.

For *this* sandbox, deferring iOS until there's a Mac available is the pragmatic call — the Cargo.toml metadata for cargo-apk lives under `[package.metadata.android.*]` and won't conflict with a future `[package.metadata.ios.*]` block.

---

## Build & deploy diagnostic toolkit

### Inspect the generated AndroidManifest

```bash
"D:/AndroidSDK/build-tools/36.0.0/aapt2.exe" dump xmltree \
    --file AndroidManifest.xml \
    target/debug/apk/<app>.apk
```

What to check:
- `activity name` is `android.app.NativeActivity` (matches what the `.so` exports)
- `meta-data android.app.lib_name` matches your `[lib] name` in Cargo.toml
- `configChanges` is `0x40007ff4` (comprehensive) or whatever set you configured
- `compileSdkVersion` and `targetSdkVersion` are sane

### Verify the activity-backend feature graph

```bash
cargo tree --target aarch64-linux-android -e features 2>&1 \
    | grep -E "android-(game|native)-activity"
```

Expected (working):
```
├── bevy feature "android-native-activity"
│   └── bevy_internal feature "android-native-activity"
│       └── bevy_winit feature "android-native-activity"
│           └── winit feature "android-native-activity"
```

If you also see `android-game-activity` in the tree, something pulled `default_platform` back in. Common culprits: `bevy/ui`, `bevy/2d`, `bevy/3d` features.

### Crash diagnosis via logcat

Always start with a clean buffer when reproducing:

```bash
adb logcat -c                                          # clear
adb shell monkey -p <package_id> -c android.intent.category.LAUNCHER 1   # launch
sleep 3
adb logcat -d 2>&1 | grep -iE \
    "FATAL|RustStdoutStderr|UnsatisfiedLink|panicked|signal [0-9]+|libhello|<your_lib_name>"
```

Useful tags:
- **`AndroidRuntime: FATAL`** — Java-side fatal exception (e.g. `UnsatisfiedLinkError` for missing native symbol)
- **`RustStdoutStderr`** — Rust panics and `eprintln!` via android-activity's stdout/stderr forwarding
- **`DEBUG`** (specifically `signal [0-9]+`) — native crash signal handler output (SIGSEGV etc.)
- **`Zygote: Process N exited due to signal 9`** — your process was reaped (often *after* a fatal error somewhere up the trace)

### App-process inspection

```bash
adb shell pidof <package_id>           # is it running? returns pid or empty
adb shell dumpsys activity activities | grep <package>   # current activity state
adb shell dumpsys gfxinfo <package_id> framestats        # rendering performance
```

### Wiping state between iterations

When migrating between build tools (e.g. xbuild → cargo-apk), the signing keystores differ. Android refuses package updates with mismatched signatures (`INSTALL_FAILED_UPDATE_INCOMPATIBLE`). Uninstall first:

```bash
adb uninstall <package_id>
```

Or pass `-r` to `adb install` only when you know signatures match (i.e. consistent build tool across runs).

### Smaller release APKs

Debug builds with full debuginfo are ~1.5 GB for a Bevy app — manageable over USB but huge. For real testing:

```bash
cargo apk run --lib --release
```

…drops to ~80–100 MB by stripping debuginfo and `opt-level=3` for the user crate. Note that the dependency crates were already at `opt-level=3` in this repo's `[profile.dev.package."*"]`, so the *runtime* speed difference between dev and release is much smaller than the size difference.

For panic traces in release, you lose Rust function symbols. Fix by building with `[profile.release] debug = "line-tables-only"` if you want stack traces without the full debuginfo blowup.

---

## Reference Cargo.toml

A minimal cargo-apk + Bevy 0.18 setup that works on Android *and* keeps `cargo run` working on desktop. Adapt the package name and feature list to your needs.

```toml
[package]
name = "my-bevy-app"
version = "0.1.0"
edition = "2024"

[lib]
name = "my_bevy_app"
crate-type = ["cdylib", "rlib"]

# Windows PDB-collision workaround (see Concepts § Hybrid lib+bin).
# Drop this on Linux/macOS-only projects.
[[bin]]
name = "my-bevy-app-bin"
path = "src/main.rs"

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3   # deps always optimized — slow first build, fast incremental

[dependencies]
# Bevy with android-game-activity swapped for android-native-activity.
# `ui`/`scene`/`audio`/`picking` omitted: `ui` re-enables default_platform
# which drags android-game-activity back in. Add what you actually use.
bevy = { version = "0.18.1", default-features = false, features = [
    # default_app
    "async_executor", "bevy_asset", "bevy_input_focus", "bevy_log",
    "bevy_state", "bevy_window", "custom_cursor", "reflect_auto_register",
    # default_platform — NativeActivity instead of GameActivity
    "std", "android-native-activity", "android_shared_stdcxx",
    "bevy_gilrs", "bevy_winit", "default_font", "multi_threaded",
    "webgl2", "x11", "wayland", "sysinfo_plugin",
    # 2d rendering pipeline (omit if your app is 3d-only)
    "2d_api", "bevy_render", "bevy_core_pipeline", "bevy_post_process",
    "bevy_sprite_render", "bevy_gizmos_render",
] }

# cargo-apk metadata: drives manifest generation, asset bundling, SDK targeting.
[package.metadata.android]
package = "com.example.my_bevy_app"
apk_name = "my-bevy-app"
assets = "assets"
build_targets = ["aarch64-linux-android"]

[package.metadata.android.sdk]
min_sdk_version = 24
target_sdk_version = 33

[package.metadata.android.application]
label = "My Bevy App"

# Comprehensive configChanges so the activity isn't recreated on
# resize/multi-window/dpi changes (which would invalidate Bevy's wgpu surface).
[package.metadata.android.application.activity]
config_changes = "orientation|keyboardHidden|keyboard|screenSize|smallestScreenSize|screenLayout|density|layoutDirection|colorMode|uiMode|locale|fontScale|navigation"
```

Plus, if your project's `.cargo/config.toml` has an `[env]` block with `CC_aarch64-linux-android` etc. set for an old xbuild setup — **delete it**. cargo-apk handles its own toolchain env vars and our overrides used to cause subtle ABI mismatches.

---

## Known gotchas — quick reference

A flat list of "things that bit me, in case they bite you", in rough order of likelihood:

1. **App crashes immediately on launch with `UnsatisfiedLinkError: undefined symbol: ANativeActivity_onCreate`** — your `.so` was built with Bevy's default `android-game-activity`. Switch to `android-native-activity` per § Bevy feature-list workaround.
2. **Spotlight/effect lands in the top-left quadrant on mobile but works on desktop** — physical/logical pixel mismatch. See § Coordinate spaces.
3. **App crashes when you maximize / multi-window resize** — incomplete `configChanges`. See § configChanges.
4. **Effect "snaps to corner" when not actively touching** — `cursor_position()` returns `Some(Vec2::ZERO)` on Android. Cfg-gate the cursor fallback.
5. **`adb install` says `INSTALL_FAILED_UPDATE_INCOMPATIBLE`** — keystore mismatch from a previous build with a different tool. `adb uninstall` first.
6. **Feature-graph compile error: "game-activity and native-activity cannot be enabled at the same time"** — you've got `bevy/ui` or another feature transitively enabling `default_platform`. See § Bevy feature-list workaround.
7. **APK is 1.5 GB** — that's debug + full debuginfo. Use `--release` for normal testing.
8. **Windows: "two output filenames will collide" warning on first build** — PDB-collision from cdylib + bin sharing the same package name. Add `[[bin]] name = "..."` override.
9. **NDK clang not found at link step** — only an issue if you bypass cargo-apk and invoke the cross-compile manually. cargo-apk handles this; xbuild doesn't.
10. **Black bars on portrait orientation / app doesn't cover whole screen** — activity isn't requesting fullscreen. Add an Android theme metadata entry like `theme = "@android:style/Theme.NoTitleBar.Fullscreen"` under `[package.metadata.android.application]`.
11. **Touch input feels sluggish or laggy compared to desktop mouse** — debug build with full debuginfo runs slow on mobile. Use `--release`.

---

## Validating a port

A quick sanity checklist when porting an existing Bevy project to Android:

- [ ] Crate-type is `["cdylib", "rlib"]`, not just the implicit bin
- [ ] `pub fn main()` in `src/lib.rs` is annotated with `#[bevy_main]`
- [ ] `cargo tree --target aarch64-linux-android -e features | grep activity` shows only `native-activity`
- [ ] All shader uniforms involving screen positions use **physical** pixels (or all use logical — but consistently)
- [ ] Touch input is read first, cursor only as desktop fallback (cfg-gated)
- [ ] `[package.metadata.android.application.activity].config_changes` covers at least `orientation|screenSize|smallestScreenSize|screenLayout|density`
- [ ] Asset paths: `AssetPlugin` filesystem override is `#[cfg(not(target_os = "android"))]`
- [ ] Pre-Bevy config files use `include_str!` on Android (or are loaded later via `AssetServer`)
- [ ] `cargo apk run --lib` (not `cargo apk run`)
- [ ] Tested on a real device (emulator works but tablet aspect ratios + scale_factor surface bugs the simulator hides)

Once all those are checked, builds tend to "just work" on subsequent ports.
