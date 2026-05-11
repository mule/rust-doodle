# bevy-ui-showcase — Phase 6: Polish & docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close out Issue #7 by updating `CLAUDE.md`'s Build Commands section, filing the deferred Android-port GitHub issue, and writing a ~100-line crate README that orients a reader through the four tabs and points at the key source files.

**Architecture:** Three artifacts, no code changes. Sequencing matters once: the GitHub issue (Artifact 3 in the spec) must land before the README (Artifact 2) so the README can link to the real issue URL. CLAUDE.md (Artifact 1) is independent of both.

**Tech Stack:** Plain Markdown + `gh` CLI. No build/check needed for any task except a final sanity confirmation that the existing crate still compiles.

---

## Notes for the executor

- Working directory: `E:\repos\rust-doodle` (Windows, PowerShell preferred for HEREDOC-style multi-line commands but Bash works too).
- Branch: `7-bevy-ui-showcase-gallery-of-bevy_ui-features`. Stay on it.
- `gh` is the GitHub CLI — already authenticated in this repo's environment (used previously to fetch Issue #7 in Phase 4 brainstorming).
- No `cargo check` / `cargo clippy` between tasks — this phase touches zero source files. Only the final task runs a quick `cargo check` to confirm we didn't break anything by accident (we shouldn't have, but the cost is ~1 second).
- Each task gets its own commit. Three commits total (CLAUDE.md, README, final validation if anything surfaces).
- The GitHub issue isn't a git commit — it's a separate artifact filed via `gh issue create`. Its issue number/URL is captured and used in Task 3.

---

## Task 1: Update `CLAUDE.md` Build Commands

**Files:**
- Modify: `CLAUDE.md`

Add the two `bevy-ui-showcase` commands to the existing "Build Commands" code block, preserving the format used by the other crates.

- [ ] **Step 1.1: Read the current `CLAUDE.md` Build Commands block.**

The relevant section currently looks like:

```bash
cargo build --manifest-path cross-platform/hello-bevy/Cargo.toml
cargo build --manifest-path cross-platform/hello-bevy-advanced/Cargo.toml
cargo build --manifest-path cross-platform/hello-rust/Cargo.toml
cargo run --manifest-path cross-platform/hello-bevy/Cargo.toml
cargo run --manifest-path cross-platform/hello-bevy-advanced/Cargo.toml
cargo build --manifest-path cross-platform/rust-poet/Cargo.toml
cargo run --manifest-path cross-platform/rust-poet/Cargo.toml -- --topic rain
cargo test --manifest-path cross-platform/rust-poet/Cargo.toml
```

- [ ] **Step 1.2: Append the two bevy-ui-showcase commands.**

Insert these two lines at the end of the code block (before the closing triple-backtick), keeping alphabetical/topical grouping consistent (they fit naturally after the `rust-poet` commands or at the bottom):

```bash
cargo build --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml
cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml
```

No `cargo test` line — the crate has no test harness (visual verification is the workflow, established in Phases 4-5).

- [ ] **Step 1.3: Verify the file still renders.**

Visually inspect the diff (`git diff CLAUDE.md`) — confirm only the two new lines added, nothing else mangled. No other section in CLAUDE.md needs changes; the Architecture section's existing notes about per-crate manifest paths, opt-level tuning, assets, and VS Code launch configs all already apply to bevy-ui-showcase.

- [ ] **Step 1.4: Commit.**

```powershell
git add CLAUDE.md
git commit -m "docs: add bevy-ui-showcase to CLAUDE.md Build Commands

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: File the Android follow-up GitHub issue

**Files:** none (creates a GitHub issue, no git commit)

Use `gh issue create` to file the Android port follow-up. The body content is fully specified — no creative writing required. Capture the issue URL returned by `gh` for use in Task 3.

- [ ] **Step 2.1: Create the issue via `gh`.**

Run from the repo root. Use a single-quoted PowerShell here-string for the body so `$` characters are not expanded:

```powershell
gh issue create --title "bevy-ui-showcase: Android port — bevy_ui vs NativeActivity collision" --body @'
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
'@
```

`gh issue create` prints the new issue's URL on success (e.g.
`https://github.com/mule/rust-doodle/issues/9`). Capture it — it goes into
Task 3's README.

- [ ] **Step 2.2: Confirm the issue exists.**

```powershell
gh issue list --search "bevy-ui-showcase Android" --json number,title,url
```

Expected: one entry matching the title you just filed. Note the `number` and `url`.

- [ ] **Step 2.3: No commit for this task.**

The GitHub issue is the artifact; nothing changes locally. Move on to Task 3.

---

## Task 3: Write the crate README

**Files:**
- Create: `cross-platform/bevy-ui-showcase/README.md`

Write a ~100-line (target ≤120) README. The content below is the verbatim text to write. Replace `URL_GOES_HERE` in the Status section with the actual issue URL captured from Task 2 (e.g. `https://github.com/mule/rust-doodle/issues/9`).

- [ ] **Step 3.1: Create the README.**

```markdown
# bevy-ui-showcase

A gallery of Bevy 0.18 UI patterns: layout, widgets, theming, and animations.
Single window, four tabs — each tab demonstrates one slice of `bevy_ui` so
the demo doubles as a quick visual reference.

## Run

From the repo root:

```bash
cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml
```

The app opens to the **Layout** tab. The tab bar at the top switches between
the four sections. The dark/light theme toggle on the right of the tab bar
reskins the whole app live (with a 300ms crossfade).

## What's inside

### Layout

Three flexbox demos. A row with `flex_grow: 1.0` on the middle child
(left- and right-pinned siblings, accordion middle). A nested column-in-row
to show axis switching. A pixel-vs-percentage demo where the percentage
boxes use `flex_grow` and a per-frame system updates their labels with the
actual percentage of the row width they currently occupy — resize the
window to watch them reflow.

### Widgets

Bevy 0.18 ships only `Button + Interaction`; every other widget here is
hand-built from `Node`s and interaction state. A click-counter button (with
hover scale-up from the animations phase), a square checkbox with a Nerd
Font checkmark glyph, a draggable slider with min/max/current labels, and
a text input with a focus-border, keyboard handler, and an emoji-glyph
popup menu.

### Theming

Centralized theme system. A `Theme` resource holds nested token structs
(bg, text, border, radius); role enum components (`BgRole`, `TextRole`,
`BorderRole`) tag every Node with the slot it wants; small resolver
systems write the resolved color. The tab itself shows a 21-swatch palette
grid, a border-radius scale (sm/md/lg/pill), a border color & width row,
and a typography sample where the heading uses Inter Bold (loaded from
`assets/fonts/`).

### Animations

Hand-rolled generic `Tween<T>` component + Bevy's built-in `EaseFunction`.
Two cross-cutting integrations: counter buttons scale 1.0 → 1.05 on hover
(150ms QuadraticOut), and toggling theme crossfades over 300ms via a
`ThemeTransition` resource the resolver systems blend through. In the tab:
a slide-in drawer (`Tween<Val::Px>` on `Node.left`), a six-curve easing
gallery (Linear / QuadraticIn / QuadraticOut / QuadraticInOut / ElasticOut
/ BackOut — race them with the Restart button), and a color crossfade
between two theme-derived swatch pairs.

## Architecture

- `src/theme.rs` — `Theme` resource, `BgTokens` / `TextTokens` /
  `BorderTokens` / `RadiusTokens`, role enum components, three resolver
  systems, `ThemeTransition` for crossfades.
- `src/tween.rs` — generic `Tween<T>` component, three advance systems
  (`f32` → `UiTransform.scale`, `Color` → `BackgroundColor`, `Val::Px` →
  `Node.left`).
- `src/nav.rs` — tab bar, theme toggle button, section visibility.
- `src/layout_section.rs` / `widgets_section.rs` / `theming_section.rs` /
  `animations_section.rs` — per-tab content modules. Each exposes a
  `spawn(commands)` function and the interaction systems it needs.
- `src/main.rs` — `App` setup, system registration, font loading.

## Bevy 0.18 specifics worth knowing

These bit us during development; calling them out so you can skip the
debugging:

- `Event` → `Message`, `EventReader` → `MessageReader`. Keyboard / mouse
  events go through `MessageReader<KeyboardInput>` etc.
- UI uses `UiTransform` / `UiGlobalTransform` (2D `Affine2`). Queries for
  the standard `Transform` / `GlobalTransform` silently match nothing on
  UI entities — this caused a "slider doesn't move" bug that took an
  embarrassing amount of time to find.
- `BorderColor` is a struct with `top/right/bottom/left` sides. Use
  `BorderColor::all(color)` for uniform borders.
- `parent_entity()` is renamed to `target_entity()` on
  `RelatedSpawnerCommands`.
- `EaseFunction::sample(t)` returns `Option<f32>` — unwrap with a fallback.

## Design docs

Full specs and implementation plans for each phase live in
`docs/superpowers/`:

- Phase 4 (Theming) — `specs/2026-05-09-bevy-ui-showcase-theming-design.md`
- Phase 5 (Animations) — `specs/2026-05-10-bevy-ui-showcase-animations-design.md`
- Phase 6 (Polish & docs) — `specs/2026-05-11-bevy-ui-showcase-polish-docs-design.md`

Phases 1-3 (crate scaffold, layout, widgets) predate the spec/plan
workflow; their commit history is the documentation.

## Status

Desktop-only. Android port is intentionally deferred — see the
[Android follow-up issue](URL_GOES_HERE). The blocker is the `bevy_ui` +
NativeActivity feature collision, documented in
`cross-platform/hello-bevy-advanced/Cargo.toml`'s feature-list comments.

## License

Inherits from the repo root.
```

CRITICAL: replace `URL_GOES_HERE` in the Status section with the real
URL captured in Task 2. Format: `[Android follow-up issue](https://github.com/mule/rust-doodle/issues/9)` (substitute the actual issue number).

- [ ] **Step 3.2: Verify line count is reasonable.**

```powershell
(Get-Content cross-platform/bevy-ui-showcase/README.md).Count
```

Expected: ~95-110 lines (the markdown above is ~96 lines before the URL
substitution; should remain under 120 after).

If the count is over 120, trim the "Bevy 0.18 specifics" bullets — that
section is the most expendable. Don't trim the per-tab paragraphs (they're
the README's reason for existing) or the architecture file list.

- [ ] **Step 3.3: Verify the issue link is present and correctly formatted.**

```powershell
Select-String -Path cross-platform/bevy-ui-showcase/README.md -Pattern "URL_GOES_HERE"
```

Expected: zero matches. If any match: the URL substitution was missed; go fix it.

```powershell
Select-String -Path cross-platform/bevy-ui-showcase/README.md -Pattern "github.com/.*/issues/\d+"
```

Expected: one match — the Android follow-up issue link.

- [ ] **Step 3.4: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/README.md
git commit -m "docs(bevy-ui-showcase): crate README orienting through the four tabs

Brief per-tab paragraphs, key file responsibilities, Bevy 0.18 gotchas
encountered, and links to the spec/plan docs in docs/superpowers/.
Mentions the Android port follow-up issue.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Final validation

**Files:** none (verification only)

A quick sanity sweep to confirm we didn't break anything and the three
artifacts all landed.

- [ ] **Step 4.1: Confirm cargo still compiles.**

The phase touched zero source files, so this should be trivially clean.

```powershell
cargo check --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml
cargo clippy --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml -- -D warnings
```

Both exit 0.

- [ ] **Step 4.2: Confirm CLAUDE.md has the new lines.**

```powershell
Select-String -Path CLAUDE.md -Pattern "bevy-ui-showcase"
```

Expected: at least 2 matches — the build and run lines.

- [ ] **Step 4.3: Confirm README exists and has no leftover placeholders.**

```powershell
Test-Path cross-platform/bevy-ui-showcase/README.md
Select-String -Path cross-platform/bevy-ui-showcase/README.md -Pattern "URL_GOES_HERE|TBD|TODO"
```

`Test-Path` returns True; `Select-String` returns nothing.

- [ ] **Step 4.4: Confirm the Android issue is filed.**

```powershell
gh issue list --search "bevy-ui-showcase Android" --json number,title
```

One entry with the matching title.

- [ ] **Step 4.5: Confirm Phase 6 is reflected in the parent Issue #7.**

This is informational only — don't close Issue #7 unilaterally. The user
may want to check the four Phase 6 checkboxes manually or close the issue
themselves after pushing.

```powershell
gh issue view 7 --json state,title
```

Expected: `state: "OPEN"`. Mention to the user that they can now check the
Phase 6 boxes (or close #7) if they're satisfied.

- [ ] **Step 4.6: No commit for this task.**

If steps 4.1-4.4 surfaced anything (e.g. URL_GOES_HERE leftover, missing
file), fix it inline and commit as `fix(bevy-ui-showcase): post-Phase-6
cleanup`. Otherwise no commit needed.

---

## Out-of-scope reminder

The spec is intentionally tight — no screenshots in the README, no
CHANGELOG file, no source-file renames or restructuring, no rewrite of
CLAUDE.md's Architecture section. If any of those come up during
implementation, file a follow-up rather than scope-creeping this plan.
