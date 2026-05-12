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
[Android follow-up issue](https://github.com/mule/rust-doodle/issues/8). The
blocker is the `bevy_ui` + NativeActivity feature collision, documented in
`cross-platform/hello-bevy-advanced/Cargo.toml`'s feature-list comments.

## License

Inherits from the repo root.
