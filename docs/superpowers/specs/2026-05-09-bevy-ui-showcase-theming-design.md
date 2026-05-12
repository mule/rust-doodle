# bevy-ui-showcase — Phase 4: Styling & theming

**Date:** 2026-05-09
**Crate:** `cross-platform/bevy-ui-showcase`
**Issue:** [#7](https://github.com/mule/rust-doodle/issues/7) Phase 4
**Branch:** `7-bevy-ui-showcase-gallery-of-bevy_ui-features`

## Goal

Demonstrate Bevy 0.18's styling and theming capabilities: a centralized
`Theme` resource with light/dark variants, a tab-bar toggle that reskins the
entire app live, and a Theming tab that teaches the system (palette swatches,
border-radius row, border-color row, typography sample with a custom font).

## Scope

**Global theming.** Every existing hardcoded `Color::srgb(...)` constant in
`nav.rs`, `layout_section.rs`, and `widgets_section.rs` migrates to a
semantic token resolved against `Res<Theme>`. Toggling theme reskins all four
tabs in real time.

Out of scope: animated theme transition (Phase 5 territory), per-component
theme overrides, theme persistence across app launches, accessibility-grade
contrast verification.

## Architecture

### Three-layer token model

```
ThemeMode (Light / Dark)
   │
   ▼
Theme resource (semantic palette: bg.surface, text.subtle, border.focus, …)
   │
   ▼
Role-component resolvers (BgRole / TextRole / BorderRole on each themed Node)
   │
   ▼
BackgroundColor / TextColor / BorderColor written on every Node
```

The split between **palette** (raw colors) and **theme** (semantic slots) is
collapsed in this iteration: the `Theme` struct holds semantic fields directly,
populated from the chosen `ThemeMode`. Adding a "raw palette" intermediate is
overengineering for a showcase — semantic fields are what UI code touches, and
that's the layer the toggle swaps.

### `Theme` resource

```rust
#[derive(Resource, Clone)]
pub struct Theme {
    pub mode: ThemeMode,
    pub bg: BgTokens,
    pub text: TextTokens,
    pub border: BorderTokens,
    pub radius: RadiusTokens,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode { Dark, Light }
```

`BgTokens` / `TextTokens` / `BorderTokens` are plain structs of `Color`
fields — one field per semantic slot (e.g. `surface`, `box_fill`, `accent`,
`button_idle`, `button_hover`, `button_pressed`, `tab_bar`, `tab_inactive`,
`tab_hovered`, `tab_active`, `input`, `emoji_btn_idle`, `emoji_btn_hover`).

`RadiusTokens` holds named radii (`sm`, `md`, `lg`, `pill`) so the
border-radius demo isn't magic numbers either.

`Theme::dark()` and `Theme::light()` are constructors. Toggle =
`*theme = if theme.mode == Dark { Theme::light() } else { Theme::dark() }`.

### Role enum components

One small enum per visual axis. Components compose by presence — a Node only
attaches the roles relevant to it.

```rust
#[derive(Component, Clone, Copy)]
pub enum BgRole {
    Background, Surface, BoxFill, Accent,
    ButtonIdle, TabBar, TabInactive,
    Input, EmojiBtnIdle,
}

#[derive(Component, Clone, Copy)]
pub enum TextRole { Primary, Subtle, OnAccent }

#[derive(Component, Clone, Copy)]
pub enum BorderRole { Subtle, Focus }
```

Hover / pressed / active variants are **not** roles — they're handled by the
existing widget interaction systems (`update_button_visuals`,
`update_tab_visuals`, `update_text_input_border`, `update_emoji_button_visuals`),
which already write `BackgroundColor` / `BorderColor` based on `Interaction`
state. Those systems gain a `theme: Res<Theme>` parameter and pull from
`theme.bg.button_hover` etc. instead of today's module-level constants.

### Resolver systems

Three small Update systems, one per role axis. Each runs when either:

1. `Res<Theme>` was mutated this frame (`theme.is_changed()`), OR
2. a Node had its role component newly inserted (`Added<BgRole>` etc.).

```rust
fn resolve_bg_role(
    theme: Res<Theme>,
    mut q: Query<(&BgRole, &mut BackgroundColor), Or<(Changed<BgRole>, Added<BgRole>)>>,
    mut q_all: Query<(&BgRole, &mut BackgroundColor)>,  // separate path on theme change
) { ... }
```

Implementation detail to settle in the plan: Bevy's query filters can't
directly express "either theme changed OR my component changed", so the
canonical pattern is two queries (or a single query plus `if theme.is_changed()`
short-circuit that does a full sweep). The spec leaves this as the implementer's
choice — both work.

### Toggle button

A new tab-bar element on the right, separate from the section tabs. Click ⇒
mutate `Res<Theme>`. Label uses Nerd Font glyphs:

- Dark mode active: ` Light` (sun icon, U+F185)
- Light mode active: ` Dark` (moon icon, U+F186)

A new system `handle_theme_toggle` watches `Interaction::Pressed` on the toggle
button and flips the resource.

### First-frame flash mitigation

Newly-spawned Nodes carry `BackgroundColor::default()` (transparent) until the
resolver runs in the next Update. At 60fps that's ~16ms — usually invisible,
sometimes visible at startup. The spec accepts this; if it's distracting in
practice, a follow-up can add spawn helpers (`themed_bg(theme, role) -> bundle`)
that pre-fill the color.

## Theming tab content

Layout matches what we agreed in brainstorming:

```
Theming & Styling

Palette
  ┌─────┬─────┬─────┬─────┬─────┬─────┐
  │ swatch grid: every BgTokens / TextTokens / BorderTokens slot,         │
  │ labeled with name and live srgb value                                  │
  └─────────────────────────────────────────────────────────────────────────┘

Border radius — sm / md / lg / pill
  Four square panels each using a different RadiusTokens entry

Border color & width
  Four bordered panels: subtle thin / subtle thick / focus thin / focus thick

Typography
  "Heading" — Inter Bold, 28px
  "Body"    — HackNerdFont, 14px (the global default)
  "Mono"    — HackNerdFont, 12px
```

The toggle button is **not** in this tab — it's in the global tab bar so the
user can flip while viewing Layout or Widgets and see them reskin.

## Custom font

Bundle **Inter** (Regular + Bold) at `assets/fonts/Inter-Regular.ttf` and
`assets/fonts/Inter-Bold.ttf`. License: SIL Open Font License 1.1 (permissive,
redistributable). Inter is proportional (HackNerdFont is monospace), so the
visual contrast in the typography sample is immediate.

Loading: lazy via `AssetServer::load("fonts/Inter-Bold.ttf")` at Theming-tab
spawn time (the rest of the app keeps the global default). No need for the
`include_bytes!` + `Assets::insert` synchronous trick we used for the global
default — Inter is only used in one place, so an async load is fine.

## Migration: where each existing constant goes

| File | Constant | New role |
|---|---|---|
| `nav.rs` | `TAB_BAR_BG` | `BgRole::TabBar` |
| `nav.rs` | `TAB_INACTIVE` | `BgRole::TabInactive` |
| `nav.rs` | `TAB_HOVERED` | (read from `theme.bg.tab_hovered` in `update_tab_visuals`) |
| `nav.rs` | `TAB_ACTIVE` | (read from `theme.bg.tab_active` in `update_tab_visuals`) |
| `nav.rs` | `TAB_TEXT` | `TextRole::Primary` |
| `layout_section.rs` | `TEXT_COLOR` | `TextRole::Primary` |
| `layout_section.rs` | `SUBTLE_COLOR` | `TextRole::Subtle` |
| `layout_section.rs` | `DEMO_BG` | `BgRole::Surface` |
| `layout_section.rs` | `BOX_COLOR` | `BgRole::BoxFill` |
| `layout_section.rs` | `ACCENT_COLOR` | `BgRole::Accent` |
| `widgets_section.rs` | `BTN_IDLE` | `BgRole::ButtonIdle` |
| `widgets_section.rs` | `BTN_HOVER` / `BTN_PRESSED` | (read from theme in `update_button_visuals`) |
| `widgets_section.rs` | `CHECKBOX_BG` | `BgRole::Input` (reuse — same role: input-ish surface) |
| `widgets_section.rs` | `CHECKBOX_BORDER` | `BorderRole::Subtle` |
| `widgets_section.rs` | `INPUT_BG` | `BgRole::Input` |
| `widgets_section.rs` | `INPUT_BORDER_IDLE` | `BorderRole::Subtle` |
| `widgets_section.rs` | `INPUT_BORDER_FOCUSED` | (read from theme in `update_text_input_border`) |
| `widgets_section.rs` | `EMOJI_BTN_IDLE` | `BgRole::EmojiBtnIdle` |
| `widgets_section.rs` | `EMOJI_BTN_HOVER` | (read from theme in `update_emoji_button_visuals`) |
| `main.rs` | `ClearColor` | small system `sync_clear_color` mirrors `theme.bg.background` into `Res<ClearColor>` when `theme.is_changed()`. The root Node also gets `BgRole::Background` so UI surfaces match the clear color exactly. |

The dark-mode color values come straight from the existing constants — no
visual change to dark mode. Light mode is a new design decision; proposed
values:

| Slot | Dark (current) | Light (new) |
|---|---|---|
| `bg.background` | `0.08, 0.09, 0.12` | `0.96, 0.96, 0.97` |
| `bg.surface` | `0.13, 0.14, 0.17` | `0.90, 0.91, 0.93` |
| `bg.box_fill` | `0.20, 0.40, 0.60` | `0.30, 0.55, 0.85` |
| `bg.accent` | `0.50, 0.30, 0.70` | `0.65, 0.45, 0.85` |
| `bg.button_idle` / hover / pressed | dark blues | lighter blues, ~+0.10 lightness step between states |
| `bg.tab_bar` | `0.10, 0.11, 0.14` | `0.92, 0.93, 0.95` |
| `text.primary` | `~0.92` | `~0.15` |
| `text.subtle` | `~0.55` | `~0.40` |
| `border.subtle` | dark mid-gray | light mid-gray |
| `border.focus` | bright accent | bright accent (same hue, slightly darker) |

Exact values get tuned visually during implementation — the spec commits to the
*structure* and the *general direction* (dark UI inverts to a light-gray UI
with darker text and the same hue families for accents).

## Validation

- [ ] `cargo build --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
- [ ] `cargo clippy --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml -- -D warnings`
- [ ] `cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
- [ ] Toggle button is visible in the tab bar.
- [ ] Clicking it flips colors across **all four tabs** with no relayout / no flicker beyond ≤1 frame.
- [ ] No raw `Color::srgb(...)` literals remain in `nav.rs`, `layout_section.rs`, or `widgets_section.rs` (only inside `theme.rs::Theme::dark()` / `::light()`).
- [ ] Theming tab renders palette swatches, radius row, border row, and typography block.
- [ ] Heading text in the typography block visibly uses Inter (proportional, distinct from the mono default).

## File changes summary

- **New:** `src/theme.rs` — `Theme`, `BgTokens`/`TextTokens`/`BorderTokens`/`RadiusTokens`, `BgRole`/`TextRole`/`BorderRole`, three resolver systems, `Theme::dark()`, `Theme::light()`, `handle_theme_toggle`.
- **New:** `src/theming_section.rs` — Theming tab content, replaces the placeholder spawned for `Section::Theming` in `main.rs`.
- **New asset:** `assets/fonts/Inter-Regular.ttf`, `assets/fonts/Inter-Bold.ttf`.
- **Modified:** `src/main.rs` — register theme resource, register resolver + toggle systems, wire `theming_section::spawn` in place of the placeholder.
- **Modified:** `src/nav.rs` — tab buttons get `BgRole::TabInactive` + `TextRole::Primary`; tab bar gets `BgRole::TabBar`; theme toggle button added; interaction systems read from `Res<Theme>`.
- **Modified:** `src/layout_section.rs` — every `Color::srgb(...)` site swaps to `(BackgroundColor::default(), BgRole::X)` or equivalent text/border role.
- **Modified:** `src/widgets_section.rs` — same migration; widget interaction systems gain `theme: Res<Theme>` parameter.

## Out of scope / follow-ups

- Animated theme transition (Phase 5 territory).
- Theme persistence across launches.
- System theme detection (follow OS dark/light setting).
- Per-component theme overrides.
- Contrast/WCAG verification of the light palette.
- Spawn helper functions (`themed_bg(theme, role)`) to eliminate the 1-frame flash. Add only if visible in practice.
