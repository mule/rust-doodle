# bevy-ui-showcase — Phase 6: Polish & docs

**Date:** 2026-05-11
**Crate:** `cross-platform/bevy-ui-showcase`
**Issue:** [#7](https://github.com/mule/rust-doodle/issues/7) Phase 6
**Branch:** `7-bevy-ui-showcase-gallery-of-bevy_ui-features`
**Predecessor:** Phase 5 (Animations) shipped at `86f10d4`.

## Goal

Close out Issue #7 with the remaining polish items: update the repo-level
`CLAUDE.md` to teach Claude about the bevy-ui-showcase crate, write a
~100-line README so a human dropping into the crate understands what each
tab demonstrates and where to dig deeper, and file the Android-port
follow-up issue documented in Issue #7's "Follow-ups" section.

## Scope

Three artifacts:

1. **`CLAUDE.md`** — add bevy-ui-showcase to the "Build Commands" section
   alongside the existing crates.
2. **`cross-platform/bevy-ui-showcase/README.md`** (new) — ~100 lines.
   Orients a reader, covers each of the four tabs briefly, points at
   `theme.rs` and `tween.rs` as the architecturally interesting files, and
   links to the spec/plan docs in `docs/superpowers/`.
3. **GitHub issue (new, filed via `gh`)** — Android port follow-up. Captures
   the bevy_ui + NativeActivity collision blocker and the two resolution
   paths.

Out of scope:
- Window title (already set in Phase 3).
- `.vscode/launch.json` entry (already added at the start of Phase 4).
- Adding screenshots to the README (text-only is fine for a sandbox).
- Rewriting CLAUDE.md's Architecture section (only Build Commands needs
  updating; Architecture is already correct).

## Artifact 1: CLAUDE.md update

Locate the existing "Build Commands" block. It currently lists hello-bevy,
hello-bevy-advanced, hello-rust, and rust-poet. Add bevy-ui-showcase commands
alongside, preserving the existing format:

```bash
cargo build --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml
cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml
```

No `cargo test` line (the crate has no test harness — Bevy UI verification
is `cargo run` + visual sweep, per the working pattern established in Phases
4-5).

No other CLAUDE.md sections need changing. The "Architecture" section's
existing notes about per-crate manifest paths, opt-level tuning, assets
under `assets/`, custom shaders, and VS Code launch configs all apply to
bevy-ui-showcase without modification.

## Artifact 2: Crate README

**Path:** `cross-platform/bevy-ui-showcase/README.md`

**Target length:** ~100 lines.

**Structure:**

```
# bevy-ui-showcase

(1-2 sentence summary: a Bevy 0.18 UI gallery covering layout, widgets,
theming, and animations.)

## Run

```bash
cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml
```

(Note about running from repo root; mention the four tabs the app opens with.)

## What's inside

### Layout (tab 1)
(2-3 sentences. Three demos: flex-row + flex_grow, nested column-in-row,
pixel-vs-percentage sizing with live percent labels.)

### Widgets (tab 2)
(2-3 sentences. Bevy ships only Button + Interaction — every other widget is
hand-built. Button with counter, checkbox, slider, text input with focus
border and emoji popup menu.)

### Theming (tab 3)
(2-3 sentences. Tab-bar toggle in the top-right reskins the whole app live.
21-swatch palette grid + border-radius row + border-color row + typography
sample using Inter Bold.)

### Animations (tab 4)
(2-3 sentences. Hand-rolled Tween<T> generic + Bevy's EaseFunction. Hover
scale-up on click-counter buttons + 300ms theme crossfade (visible across
the app). Local demos: slide-in drawer, six-curve easing gallery, color
crossfade.)

## Architecture

(One paragraph naming the three key files and their responsibilities.)

- `src/theme.rs` — `Theme` resource with semantic token structs, role
  enum components, three resolver systems, ThemeTransition for crossfades.
- `src/tween.rs` — generic `Tween<T>` component, advance systems for f32 /
  Color / Val::Px.
- `src/nav.rs` — tab bar and section visibility.
- Per-tab content modules: `layout_section.rs`, `widgets_section.rs`,
  `theming_section.rs`, `animations_section.rs`.

## Bevy 0.18 specifics encountered

(Brief bullet list — the gotchas that bit us during development.)

- `Event` → `Message`, `EventReader` → `MessageReader` (input handling).
- UI uses `UiTransform` / `UiGlobalTransform` — not the standard
  `Transform` / `GlobalTransform`. Failed queries for the wrong type
  silently return nothing.
- `BorderColor` is a struct with `top/right/bottom/left` sides — use
  `BorderColor::all(color)`.
- `parent_entity()` is now `target_entity()` on `RelatedSpawnerCommands`.

## Design docs

The full design/spec/plan documents for each phase live in
`docs/superpowers/specs/` and `docs/superpowers/plans/`:

- Phase 4 (Theming): `2026-05-09-bevy-ui-showcase-theming-design.md`
- Phase 5 (Animations): `2026-05-10-bevy-ui-showcase-animations-design.md`

Earlier phases (1-3 — crate scaffold, layout, widgets) were brainstormed
informally; their commit history is the documentation.

## Status

Desktop-only. Android port is intentionally deferred — see
[follow-up issue](TBD-link-after-filing). The blocker is documented in
`cross-platform/hello-bevy-advanced/Cargo.toml`'s feature-list comments.

## License

(Inherits from the repo root.)
```

The README MUST stay under ~120 lines. If a section pushes past, trim
prose, not links — links are the README's load-bearing element.

The "TBD-link-after-filing" placeholder gets replaced with the actual issue
URL once Artifact 3 is filed (the implementation plan will sequence the
issue filing before the README write so the URL is available).

## Artifact 3: Android follow-up GitHub issue

**Filed via:** `gh issue create --title "..." --body "..."`

**Title:** `bevy-ui-showcase: Android port — bevy_ui vs NativeActivity collision`

**Body structure:**

```
## Summary

Port the desktop-only bevy-ui-showcase crate to Android. Blocked on a known
Bevy 0.18 feature-list collision between `bevy_ui` and the NativeActivity
backend this repo uses for `hello-bevy-advanced`.

## Blocker

The repo's `hello-bevy-advanced` crate uses `cargo-apk` with NativeActivity,
configured via a hand-curated Bevy feature list that excludes
`bevy/default_platform` (see `cross-platform/hello-bevy-advanced/Cargo.toml`
feature-list comments).

Enabling `bevy_ui` re-pulls `bevy/default_platform`, which transitively
drags in `bevy_winit/android-game-activity`. This conflicts with the
NativeActivity backend (`android-native-activity`) that the rest of the
build is set up for.

## Resolution paths

1. **Hand-curated bevy_ui feature list** — figure out the minimal set of
   `bevy` features needed for `bevy_ui` to function without re-pulling
   `default_platform`. Likely involves enabling specific UI sub-features
   directly. Stays on NativeActivity. Less work for `cargo-apk` config.
   
2. **Switch this crate to GameActivity** — use a separate `cargo-apk`
   config (`AndroidManifest.xml`, Java glue) that targets GameActivity
   instead of NativeActivity. Lets us pull `default_platform` cleanly.
   Diverges from the rest of the repo's Android setup.

Path 1 is preferred if it works because it keeps the toolchain consistent
across crates. Path 2 is the fallback if Bevy 0.18's bevy_ui doesn't
support being decoupled from `default_platform`.

## References

- `cross-platform/hello-bevy-advanced/Cargo.toml` — existing feature-list
  workaround, with comments explaining the collision.
- `docs/bevy-android.md` — Bevy-on-Android working-developer notes
  (coordinate spaces, activity-backend gotcha, diagnostic toolkit).
- Parent issue: #7 (Phase 6 "Follow-ups" section explicitly references
  this).

## Acceptance criteria

- `cargo apk run --lib` (from `cross-platform/bevy-ui-showcase/`) launches
  the showcase on a connected Android device or emulator.
- All four tabs render correctly on Android touch input.
- Resolution and DPI on Android don't break the layout demos.
- The Android-specific code path is `cfg(target_os = "android")`-gated; the
  desktop binary continues to build via `cargo run --manifest-path ...`.
```

The issue is opened via `gh issue create` so it gets a real issue number
that the README's "follow-up issue" link can reference.

## Sequencing constraint

Artifact 3 (file the GitHub issue) MUST happen before Artifact 2 (write the
README), so the README can reference the real issue URL.

CLAUDE.md (Artifact 1) is independent and can happen in any order.

## Validation

- [ ] `CLAUDE.md` contains the two new `bevy-ui-showcase` lines under
      "Build Commands".
- [ ] `cross-platform/bevy-ui-showcase/README.md` exists, is ≤120 lines,
      and renders cleanly on GitHub (manual visual check of the rendered
      Markdown).
- [ ] GitHub issue exists, returned by `gh issue list --search
      "bevy-ui-showcase Android"`, body contains the resolution-paths
      section.
- [ ] README's "follow-up issue" link points at the real issue URL (not
      `TBD-link-after-filing`).

## Out of scope / follow-ups

- Screenshots in the README.
- A CHANGELOG file (sandbox project; git log is sufficient).
- Renaming or restructuring existing source files for clarity.
- Any cleanup of pre-Phase-4 code (the migrations in Phase 4 already
  rewrote everything touchable).
