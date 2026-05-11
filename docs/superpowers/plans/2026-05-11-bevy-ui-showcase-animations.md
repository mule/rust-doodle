# bevy-ui-showcase — Phase 5: Animations & transitions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a hand-rolled generic `Tween<T>` animation system to the bevy-ui-showcase, then apply it across two cross-cutting integrations (theme toggle crossfade, hover scale-up on counter buttons) plus four discrete demos in a new Animations tab.

**Architecture:** New `src/tween.rs` defines the `Tween<T>` component generic with three concrete advance systems (`f32` for scale, `Color` for crossfades, `Val::Px` for slide positions). Theme crossfade uses a transient `ThemeTransition` resource snapshotted on toggle; the existing role resolvers gain a blend branch. Hover scale-up is a parallel system to `update_button_visuals` that inserts a `Tween<f32>` on `Changed<Interaction>`. The Animations tab content lives in a new `src/animations_section.rs`. Zero new dependencies.

**Tech Stack:** Bevy 0.18.1, `bevy_math::EaseFunction` for curves, no test harness (verification is `cargo check` + `cargo run` visual sweep).

---

## Notes for the executor

- This crate has **no automated test suite**. "Verify" steps are either `cargo check`, `cargo clippy -- -D warnings`, or `cargo run` (visual). Visual sweep falls to the user; the implementer only confirms static checks.
- Bevy 0.18.1 specifics already known in this codebase: `Event` → `Message`, `EventReader` → `MessageReader`, `parent_entity()` → `target_entity()`, UI uses `UiTransform`/`UiGlobalTransform`, `BorderColor` is a struct with `top/right/bottom/left` sides (use `BorderColor::all(color)`).
- `EaseFunction` API location: in Bevy 0.18.1 it's at `bevy::math::curve::EaseFunction`. If the implementer's editor can't resolve it there, try `bevy::math::EaseFunction` (top-level re-export). Variants used: `Linear`, `QuadraticIn`, `QuadraticOut`, `QuadraticInOut`, `ElasticOut`, `BackOut`.
- The plan calls `easing.sample(t)` returning `Option<f32>`, with `.unwrap_or(t)` falling back to linear if a curve fails to sample. If the actual Bevy 0.18.1 API differs (e.g., returns `f32` directly without `Option`, or requires wrapping in `EasingCurve::new(0.0, 1.0, ease).sample(t)`), adapt the call site. The rest of the plan still applies.
- `f32::lerp(self, other, t) -> f32` is stable in Rust 1.85 (the crate uses edition 2024 → requires 1.85+).
- `Color` implements `bevy_color::Mix`: `color.mix(&other, t) -> Color`.
- `cargo clippy -- -D warnings` is the validation gate — it passed at the end of Phase 4 and must pass at the end of Phase 5. Any new system with a complex `Query<...>` filter may need `#[allow(clippy::type_complexity)]` (canonical Bevy idiom).
- Build/check commands always use `--manifest-path cross-platform/bevy-ui-showcase/Cargo.toml` from the repo root (`E:\repos\rust-doodle`).
- Branch: `7-bevy-ui-showcase-gallery-of-bevy_ui-features`. Stay on it.

---

## Task 1: Create `tween.rs` — the Tween primitive

**Files:**
- Create: `cross-platform/bevy-ui-showcase/src/tween.rs`
- Modify: `cross-platform/bevy-ui-showcase/src/main.rs` (add `mod tween;`, register three advance systems)

This task ships the entire tween infrastructure as one self-contained module: the generic `Tween<T>` component and three concrete advance systems for `f32`, `Color`, and `Val`. Nothing in the rest of the app uses tweens yet — that arrives in later tasks. The module compiles in isolation and contributes no observable behavior.

- [ ] **Step 1.1: Create `src/tween.rs`.**

```rust
use bevy::math::curve::EaseFunction;
use bevy::prelude::*;

/// Generic tween component. Each concrete `T` is its own ECS-distinct
/// component type — `Tween<f32>` and `Tween<Color>` are unrelated.
/// Three concrete advance systems below cover the property types this
/// crate animates: `f32` for scale, `Color` for crossfades, `Val` for
/// slide positions.
#[derive(Component)]
pub struct Tween<T: Clone + Send + Sync + 'static> {
    pub start: T,
    pub end: T,
    pub elapsed: f32,
    pub duration: f32,
    pub easing: EaseFunction,
}

/// Advance every `Tween<f32>` and write the interpolated value into the
/// entity's `UiTransform.scale` (uniform x and y). Removes the tween
/// component when complete.
pub fn advance_f32_tweens(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Tween<f32>, &mut UiTransform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tween, mut transform) in &mut q {
        tween.elapsed += dt;
        let raw = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
        let eased = tween.easing.sample(raw).unwrap_or(raw);
        let value = tween.start.lerp(tween.end, eased);
        transform.scale = Vec2::splat(value);
        if raw >= 1.0 {
            commands.entity(entity).remove::<Tween<f32>>();
        }
    }
}

/// Advance every `Tween<Color>` and write the mixed color into the
/// entity's `BackgroundColor`. Removes the tween component when complete.
pub fn advance_color_tweens(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Tween<Color>, &mut BackgroundColor)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tween, mut bg) in &mut q {
        tween.elapsed += dt;
        let raw = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
        let eased = tween.easing.sample(raw).unwrap_or(raw);
        bg.0 = tween.start.mix(&tween.end, eased);
        if raw >= 1.0 {
            commands.entity(entity).remove::<Tween<Color>>();
        }
    }
}

/// Advance every `Tween<Val>` and write the interpolated value into the
/// entity's `Node.left`. `start` and `end` must both be `Val::Px(_)`;
/// debug builds assert, release falls back to treating other variants as 0px.
pub fn advance_val_tweens(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Tween<Val>, &mut Node)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tween, mut node) in &mut q {
        tween.elapsed += dt;
        let raw = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
        let eased = tween.easing.sample(raw).unwrap_or(raw);
        let start_px = px_of(tween.start);
        let end_px = px_of(tween.end);
        node.left = Val::Px(start_px.lerp(end_px, eased));
        if raw >= 1.0 {
            commands.entity(entity).remove::<Tween<Val>>();
        }
    }
}

fn px_of(v: Val) -> f32 {
    match v {
        Val::Px(x) => x,
        other => {
            debug_assert!(false, "Tween<Val> requires Val::Px, got {other:?}");
            0.0
        }
    }
}
```

- [ ] **Step 1.2: Wire into `main.rs`.**

Add `mod tween;` near the top alongside the other module declarations. Append `tween::advance_f32_tweens`, `tween::advance_color_tweens`, `tween::advance_val_tweens` to the second `.add_systems(Update, (...))` block (the one with the theme systems).

- [ ] **Step 1.3: Verify it compiles.**

Run: `cargo check --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
Expected: zero errors. There will be unused-warning noise on `Tween<T>` because nothing inserts one yet — that's expected and resolved by Tasks 2-8.

If `EaseFunction` doesn't resolve at `bevy::math::curve::EaseFunction`, try `bevy::math::EaseFunction` or look up the import in the Bevy 0.18.1 docs.

If `easing.sample(t)` returns something other than `Option<f32>`, adapt the call site (the API may have changed; the plan's `.unwrap_or(raw)` is for the `Option<f32>` shape).

- [ ] **Step 1.4: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/tween.rs cross-platform/bevy-ui-showcase/src/main.rs
git commit -m "feat(bevy-ui-showcase): Tween<T> primitive + three advance systems

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: Theme transition infrastructure

**Files:**
- Modify: `cross-platform/bevy-ui-showcase/src/theme.rs`
- Modify: `cross-platform/bevy-ui-showcase/src/main.rs` (register one new system)

Add the `ThemeTransition` resource, `mix` methods on the three token structs, and the `advance_theme_transition` system that ticks the resource and removes it when complete. The three resolver systems gain a blend branch but it's never reached yet (the resource is only inserted by Task 3's wire-up). Compiles in isolation; no observable behavior change.

- [ ] **Step 2.1: Add `ThemeTransition` resource at the top of `theme.rs`, after the existing `Theme` declarations.**

Add the import for `EaseFunction` near the top of `theme.rs` (alongside existing `use bevy::prelude::*;`):

```rust
use bevy::math::curve::EaseFunction;
```

Add the resource type after the `Default for Theme` impl:

```rust
/// Active while the app is mid-crossfade between dark and light. Snapshotted
/// by `handle_theme_toggle` at the moment of the click; advanced by
/// `advance_theme_transition`; consumed by the three role resolvers and
/// `sync_clear_color` as a per-frame blend source.
#[derive(Resource)]
pub struct ThemeTransition {
    pub from_bg: BgTokens,
    pub from_text: TextTokens,
    pub from_border: BorderTokens,
    pub elapsed: f32,
    pub duration: f32,
    pub easing: EaseFunction,
}

impl ThemeTransition {
    pub fn eased_progress(&self) -> f32 {
        let raw = (self.elapsed / self.duration).clamp(0.0, 1.0);
        self.easing.sample(raw).unwrap_or(raw)
    }
}
```

- [ ] **Step 2.2: Add `mix` impls on the three token structs.**

After each existing `impl BgTokens { ... }` / `TextTokens` / `BorderTokens` resolver block (or below their struct declarations), add:

```rust
impl BgTokens {
    pub fn mix(a: &BgTokens, b: &BgTokens, t: f32) -> BgTokens {
        use bevy::color::Mix;
        BgTokens {
            background: a.background.mix(&b.background, t),
            surface: a.surface.mix(&b.surface, t),
            box_fill: a.box_fill.mix(&b.box_fill, t),
            accent: a.accent.mix(&b.accent, t),
            button_idle: a.button_idle.mix(&b.button_idle, t),
            button_hover: a.button_hover.mix(&b.button_hover, t),
            button_pressed: a.button_pressed.mix(&b.button_pressed, t),
            tab_bar: a.tab_bar.mix(&b.tab_bar, t),
            tab_inactive: a.tab_inactive.mix(&b.tab_inactive, t),
            tab_hovered: a.tab_hovered.mix(&b.tab_hovered, t),
            tab_active: a.tab_active.mix(&b.tab_active, t),
            input: a.input.mix(&b.input, t),
            emoji_btn_idle: a.emoji_btn_idle.mix(&b.emoji_btn_idle, t),
            emoji_btn_hover: a.emoji_btn_hover.mix(&b.emoji_btn_hover, t),
            slider_track: a.slider_track.mix(&b.slider_track, t),
            slider_thumb: a.slider_thumb.mix(&b.slider_thumb, t),
        }
    }
}

impl TextTokens {
    pub fn mix(a: &TextTokens, b: &TextTokens, t: f32) -> TextTokens {
        use bevy::color::Mix;
        TextTokens {
            primary: a.primary.mix(&b.primary, t),
            subtle: a.subtle.mix(&b.subtle, t),
            on_accent: a.on_accent.mix(&b.on_accent, t),
        }
    }
}

impl BorderTokens {
    pub fn mix(a: &BorderTokens, b: &BorderTokens, t: f32) -> BorderTokens {
        use bevy::color::Mix;
        BorderTokens {
            subtle: a.subtle.mix(&b.subtle, t),
            focus: a.focus.mix(&b.focus, t),
        }
    }
}
```

(If `bevy::color::Mix` doesn't resolve, try `bevy::prelude::Mix` or `bevy_color::Mix`. The trait exists in Bevy 0.18.1.)

- [ ] **Step 2.3: Modify the three resolver systems to blend during transition.**

Replace `resolve_bg_role`:

```rust
pub fn resolve_bg_role(
    theme: Res<Theme>,
    transition: Option<Res<ThemeTransition>>,
    mut q: Query<(&BgRole, &mut BackgroundColor)>,
    added: Query<(), Added<BgRole>>,
) {
    let mid_transition = transition.is_some();
    if !theme.is_changed() && added.is_empty() && !mid_transition {
        return;
    }
    let blended_bg = if let Some(t) = transition.as_ref() {
        BgTokens::mix(&t.from_bg, &theme.bg, t.eased_progress())
    } else {
        theme.bg
    };
    for (role, mut bg) in &mut q {
        bg.0 = blended_bg.resolve(*role);
    }
}
```

Replace `resolve_text_role`:

```rust
pub fn resolve_text_role(
    theme: Res<Theme>,
    transition: Option<Res<ThemeTransition>>,
    mut q: Query<(&TextRole, &mut TextColor)>,
    added: Query<(), Added<TextRole>>,
) {
    let mid_transition = transition.is_some();
    if !theme.is_changed() && added.is_empty() && !mid_transition {
        return;
    }
    let blended_text = if let Some(t) = transition.as_ref() {
        TextTokens::mix(&t.from_text, &theme.text, t.eased_progress())
    } else {
        theme.text
    };
    for (role, mut color) in &mut q {
        color.0 = blended_text.resolve(*role);
    }
}
```

Replace `resolve_border_role`:

```rust
pub fn resolve_border_role(
    theme: Res<Theme>,
    transition: Option<Res<ThemeTransition>>,
    mut q: Query<(&BorderRole, &mut BorderColor)>,
    added: Query<(), Added<BorderRole>>,
) {
    let mid_transition = transition.is_some();
    if !theme.is_changed() && added.is_empty() && !mid_transition {
        return;
    }
    let blended_border = if let Some(t) = transition.as_ref() {
        BorderTokens::mix(&t.from_border, &theme.border, t.eased_progress())
    } else {
        theme.border
    };
    for (role, mut color) in &mut q {
        *color = BorderColor::all(blended_border.resolve(*role));
    }
}
```

- [ ] **Step 2.4: Modify `sync_clear_color` to blend during transition.**

```rust
pub fn sync_clear_color(
    theme: Res<Theme>,
    transition: Option<Res<ThemeTransition>>,
    mut clear: ResMut<ClearColor>,
) {
    let mid_transition = transition.is_some();
    if !theme.is_changed() && !mid_transition {
        return;
    }
    clear.0 = if let Some(t) = transition.as_ref() {
        use bevy::color::Mix;
        t.from_bg.background.mix(&theme.bg.background, t.eased_progress())
    } else {
        theme.bg.background
    };
}
```

- [ ] **Step 2.5: Add `advance_theme_transition` system at the bottom of `theme.rs`.**

```rust
pub fn advance_theme_transition(
    time: Res<Time>,
    mut commands: Commands,
    transition: Option<ResMut<ThemeTransition>>,
) {
    let Some(mut t) = transition else { return; };
    t.elapsed += time.delta_secs();
    if t.elapsed >= t.duration {
        commands.remove_resource::<ThemeTransition>();
    }
}
```

- [ ] **Step 2.6: Register `theme::advance_theme_transition` in `main.rs`.**

Add to the second `.add_systems(Update, (...))` tuple alongside the other theme systems.

- [ ] **Step 2.7: Verify it compiles.**

Run: `cargo check --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
Expected: zero errors. New unused-warning noise on `ThemeTransition` is expected — Task 3 inserts it.

- [ ] **Step 2.8: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/theme.rs cross-platform/bevy-ui-showcase/src/main.rs
git commit -m "feat(bevy-ui-showcase): theme transition resource + blending resolvers

Adds ThemeTransition resource, mix() helpers on token structs, and an
advance system. The three role resolvers and sync_clear_color now blend
between captured 'from' tokens and the live theme during transition.
Nothing inserts ThemeTransition yet — that wires up in the next task.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Wire up theme toggle crossfade

**Files:**
- Modify: `cross-platform/bevy-ui-showcase/src/theme.rs` (one function)

Modify `handle_theme_toggle` to snapshot the current token structs into a new `ThemeTransition` resource at the moment of the click. After this task, clicking the theme toggle button produces a visible 300ms crossfade instead of a snap.

- [ ] **Step 3.1: Replace the existing `handle_theme_toggle` in `theme.rs`.**

```rust
pub fn handle_theme_toggle(
    mut commands: Commands,
    mut theme: ResMut<Theme>,
    transition: Option<Res<ThemeTransition>>,
    q: Query<&Interaction, (Changed<Interaction>, With<ThemeToggle>)>,
) {
    for interaction in &q {
        if *interaction == Interaction::Pressed && transition.is_none() {
            let from_bg = theme.bg;
            let from_text = theme.text;
            let from_border = theme.border;
            theme.toggle();
            commands.insert_resource(ThemeTransition {
                from_bg,
                from_text,
                from_border,
                elapsed: 0.0,
                duration: 0.3,
                easing: EaseFunction::QuadraticInOut,
            });
        }
    }
}
```

The `transition.is_none()` guard prevents stacking transitions during rapid clicks — if a fade is in flight, the click is ignored.

- [ ] **Step 3.2: Verify it compiles.**

Run: `cargo check --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
Expected: zero errors. The `ThemeTransition` unused-warning from Task 2 should clear.

- [ ] **Step 3.3: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/theme.rs
git commit -m "feat(bevy-ui-showcase): theme toggle crossfades over 300ms QuadraticInOut

handle_theme_toggle now snapshots the outgoing token structs and inserts
a ThemeTransition resource alongside the theme.toggle() call. The resolver
systems pick up the resource on the next frame and start blending. Rapid
re-clicks are ignored while a transition is in flight.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Hover scale-tween on ClickCount buttons

**Files:**
- Modify: `cross-platform/bevy-ui-showcase/src/widgets_section.rs`
- Modify: `cross-platform/bevy-ui-showcase/src/main.rs`

Add a new system `dispatch_button_hover_scale` that inserts a `Tween<f32>` on `Changed<Interaction>` for buttons carrying `ClickCount`. The existing `update_button_visuals` (which writes color directly) is unchanged. Both systems run independently — color stays instant, scale tweens.

- [ ] **Step 4.1: Add the new system at the bottom of `widgets_section.rs`.**

Add these imports at the top of the file if not already present:

```rust
use bevy::math::curve::EaseFunction;
use crate::tween::Tween;
```

Append the system:

```rust
/// Insert a `Tween<f32>` on `UiTransform.scale` whenever a `ClickCount`
/// button's `Interaction` changes. Reads the current scale (mid-tween or
/// not) so a fast re-hover continues smoothly from the visible position
/// rather than snapping to 1.0.
///
/// Tab buttons / slider tracks / emoji buttons are NOT targeted (filter
/// uses `With<ClickCount>`), so chrome stays instant.
#[allow(clippy::type_complexity)]
pub fn dispatch_button_hover_scale(
    mut commands: Commands,
    q: Query<
        (Entity, &Interaction, &UiTransform),
        (Changed<Interaction>, With<ClickCount>),
    >,
) {
    for (entity, interaction, transform) in &q {
        let current = transform.scale.x;
        let (target, duration) = match *interaction {
            Interaction::Hovered => (1.05, 0.15),
            Interaction::None => (1.0, 0.15),
            Interaction::Pressed => (0.97, 0.08),
        };
        commands.entity(entity).insert(Tween::<f32> {
            start: current,
            end: target,
            elapsed: 0.0,
            duration,
            easing: EaseFunction::QuadraticOut,
        });
    }
}
```

- [ ] **Step 4.2: Register `widgets_section::dispatch_button_hover_scale` in `main.rs`.**

Add to one of the existing `.add_systems(Update, (...))` tuples (whichever has room — the first tuple has 18 systems and may need to spill into the second; check current capacity). The order doesn't matter for correctness; it can run before or after `update_button_visuals`.

- [ ] **Step 4.3: Verify it compiles and clippy passes.**

```powershell
cargo check --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml
cargo clippy --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml -- -D warnings
```

Both should exit 0. The `Tween<f32>` unused-warning from Task 1 should clear.

- [ ] **Step 4.4: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/widgets_section.rs cross-platform/bevy-ui-showcase/src/main.rs
git commit -m "feat(bevy-ui-showcase): hover scale-tween on click-counter buttons

dispatch_button_hover_scale inserts a Tween<f32> on UiTransform.scale
whenever a ClickCount button's Interaction changes. Hovered=1.05/150ms,
None=1.0/150ms, Pressed=0.97/80ms, all QuadraticOut. Tab buttons,
slider tracks, and emoji buttons stay instant (filter is With<ClickCount>).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Animations section scaffold + Block 1 (Hover scale-up)

**Files:**
- Create: `cross-platform/bevy-ui-showcase/src/animations_section.rs`
- Modify: `cross-platform/bevy-ui-showcase/src/main.rs`

Create the new section module with header + content column + a single block that demos hover scale-up (one `ClickCount` button — the global hover-scale system from Task 4 applies). Replace the existing `Section::Animations` placeholder spawn in main.rs.

- [ ] **Step 5.1: Create `src/animations_section.rs`.**

```rust
use bevy::prelude::*;

use crate::nav::{Section, SectionRoot};
use crate::theme::{BgRole, BorderRole, TextRole};
use crate::widgets_section::{ClickCount, ClickCountLabel};

pub fn spawn(commands: &mut Commands) -> Entity {
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                display: Display::None,
                ..default()
            },
            SectionRoot(Section::Animations),
        ))
        .id();

    let header = commands
        .spawn(Node {
            padding: UiRect::all(Val::Px(24.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|h| {
            h.spawn((
                Text::new("Animations & Transitions"),
                TextFont { font_size: 32.0, ..default() },
                TextColor::default(),
                TextRole::Primary,
            ));
            h.spawn((
                Text::new(
                    "Hand-rolled Tween<T> generic + Bevy's EaseFunction. \
                     Theme toggling and counter-button hover are already \
                     animated globally — the demos below show the machinery.",
                ),
                TextColor::default(),
                TextRole::Subtle,
            ));
        })
        .id();

    let content = commands
        .spawn(Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(24.0)),
            row_gap: Val::Px(24.0),
            ..default()
        })
        .with_children(|c| {
            // ── Block 1: Hover scale-up ──
            c.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|cell| {
                cell.spawn((
                    Text::new("Hover scale-up"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor::default(),
                    TextRole::Primary,
                ));
                cell.spawn((
                    Text::new(
                        "Scales 1.0 → 1.05 over 150ms (QuadraticOut). \
                         Color stays instant — see dispatch_button_hover_scale.",
                    ),
                    TextColor::default(),
                    TextRole::Subtle,
                ));
                cell.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        padding: UiRect::all(Val::Px(12.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor::default(),
                    BgRole::Surface,
                ))
                .with_children(|row| {
                    row.spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor::default(),
                        BgRole::ButtonIdle,
                        ClickCount::default(),
                    ))
                    .with_child((
                        Text::new("Hover me"),
                        TextColor::default(),
                        TextRole::Primary,
                        ClickCountLabel,
                    ));
                });
            });
        })
        .id();

    commands.entity(root).add_children(&[header, content]);
    root
}
```

(Notes: `ClickCount` and `ClickCountLabel` are already `pub(crate)` in `widgets_section.rs` per the Phase 4 codebase. The `ClickCountLabel` on the child Text means the existing `update_click_buttons` system will rewrite the label on click — bonus integration, not the focus of this block. The hover scale comes from `dispatch_button_hover_scale` automatically.)

- [ ] **Step 5.2: Wire `animations_section::spawn` into `main.rs`.**

Near the top of `main.rs`, alongside the other `mod` decls:

```rust
mod animations_section;
```

Inside `setup_root`, replace the `spawn_placeholder(&mut commands, Section::Animations)` line with:

```rust
sections.push(animations_section::spawn(&mut commands));
```

The `spawn_placeholder` function and its `spawn_placeholder(&mut commands, Section::Animations)` call should be removed entirely now that all four sections have real implementations. If `spawn_placeholder` is still in the file but no longer called, delete its definition too. Otherwise `cargo check` will warn about unused code (which `-D warnings` flips to an error).

- [ ] **Step 5.3: Verify.**

```powershell
cargo check --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml
cargo clippy --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml -- -D warnings
```

Both exit 0.

- [ ] **Step 5.4: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/animations_section.rs cross-platform/bevy-ui-showcase/src/main.rs
git commit -m "feat(bevy-ui-showcase): animations section scaffold + hover scale demo

Replaces the Section::Animations placeholder with a real module containing
the header + first block (Hover scale-up). The button is a ClickCount
entity so the global dispatch_button_hover_scale picks it up automatically.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: Block 2 — Slide-in panel

**Files:**
- Modify: `cross-platform/bevy-ui-showcase/src/animations_section.rs`
- Modify: `cross-platform/bevy-ui-showcase/src/main.rs` (register one new system)

Add a demo container with `overflow: clip`, an off-screen panel that slides in from the right on toggle, and the button that triggers it.

The panel is positioned via `Node.left` (not `Node.right`) because `advance_val_tweens` writes to `left`. Inside a 600-wide container, `left: 600.0` = panel fully off-screen to the right (panel's left edge sits at container's right edge); `left: 280.0` = panel right-aligned inside container (panel width 320 + left 280 = 600 = container width).

- [ ] **Step 6.1: Add components and the toggle system to `animations_section.rs`.**

After the existing `spawn` function, add:

```rust
use bevy::math::curve::EaseFunction;
use crate::tween::Tween;

/// Marker on the panel that slides in/out.
#[derive(Component)]
pub(crate) struct DrawerPanel;

/// Open-state on the panel (initially false = off-screen).
#[derive(Component, Default)]
pub(crate) struct DrawerOpen(pub bool);

/// Marker on the button that toggles the drawer.
#[derive(Component)]
pub(crate) struct DrawerToggle;

/// On `Pressed` of a DrawerToggle button, flip every DrawerOpen's bool and
/// insert a fresh Tween<Val> that animates the panel's `left` property.
/// (Multiple panels could exist; in practice there's one.)
#[allow(clippy::type_complexity)]
pub fn toggle_drawer(
    mut commands: Commands,
    buttons: Query<&Interaction, (Changed<Interaction>, With<DrawerToggle>)>,
    mut panels: Query<(Entity, &Node, &mut DrawerOpen), With<DrawerPanel>>,
) {
    let mut clicked = false;
    for interaction in &buttons {
        if *interaction == Interaction::Pressed {
            clicked = true;
        }
    }
    if !clicked {
        return;
    }
    for (entity, node, mut open) in &mut panels {
        open.0 = !open.0;
        let current = match node.left {
            Val::Px(v) => v,
            _ => 0.0,
        };
        let (target, duration, easing) = if open.0 {
            (280.0, 0.25, EaseFunction::QuadraticOut)
        } else {
            (600.0, 0.20, EaseFunction::QuadraticIn)
        };
        commands.entity(entity).insert(Tween::<Val> {
            start: Val::Px(current),
            end: Val::Px(target),
            elapsed: 0.0,
            duration,
            easing,
        });
    }
}
```

- [ ] **Step 6.2: Add the Block 2 spawn to the content closure in `spawn`.**

Inside the `with_children(|c| { ... })` closure on the content column, AFTER the existing Block 1 spawn, add:

```rust
            // ── Block 2: Slide-in panel ──
            c.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|cell| {
                cell.spawn((
                    Text::new("Slide-in panel"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor::default(),
                    TextRole::Primary,
                ));
                cell.spawn((
                    Text::new(
                        "Click the toggle. A 320px-wide panel slides in from \
                         the right edge of the demo area; click again to \
                         slide it out. Uses Tween<Val::Px> on Node.left.",
                    ),
                    TextColor::default(),
                    TextRole::Subtle,
                ));

                // Demo container — fixed-size with overflow: clip so the
                // off-screen panel doesn't bleed into surrounding layout.
                cell.spawn((
                    Node {
                        width: Val::Px(600.0),
                        height: Val::Px(300.0),
                        position_type: PositionType::Relative,
                        overflow: Overflow::clip(),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor::default(),
                    BgRole::Surface,
                ))
                .with_children(|container| {
                    // The panel — absolutely positioned, initially off-screen
                    // (left: 600 = panel left-edge at container right-edge).
                    container
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(0.0),
                                bottom: Val::Px(0.0),
                                left: Val::Px(600.0),
                                width: Val::Px(320.0),
                                padding: UiRect::all(Val::Px(16.0)),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(8.0),
                                border: UiRect::left(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor::default(),
                            BgRole::Input,
                            BorderColor::default(),
                            BorderRole::Subtle,
                            DrawerPanel,
                            DrawerOpen::default(),
                        ))
                        .with_children(|panel| {
                            panel.spawn((
                                Text::new("Drawer"),
                                TextFont { font_size: 20.0, ..default() },
                                TextColor::default(),
                                TextRole::Primary,
                            ));
                            panel.spawn((
                                Text::new("Placeholder content for the demo."),
                                TextColor::default(),
                                TextRole::Subtle,
                            ));
                        });
                });

                // Toggle button below the container.
                cell.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor::default(),
                    BgRole::ButtonIdle,
                    DrawerToggle,
                ))
                .with_child((
                    Text::new("Toggle drawer"),
                    TextColor::default(),
                    TextRole::Primary,
                ));
            });
```

- [ ] **Step 6.3: Register `animations_section::toggle_drawer` in `main.rs`.**

Add to one of the `.add_systems(Update, (...))` tuples.

- [ ] **Step 6.4: Verify.**

```powershell
cargo check --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml
cargo clippy --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml -- -D warnings
```

Both exit 0.

- [ ] **Step 6.5: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/animations_section.rs cross-platform/bevy-ui-showcase/src/main.rs
git commit -m "feat(bevy-ui-showcase): slide-in drawer demo (Block 2)

A 600x300 demo container with overflow: clip holds a 320px-wide panel
absolutely positioned at left: 600px initially (panel left-edge at
container right-edge). Clicking the Toggle drawer button flips DrawerOpen
and inserts a Tween<Val> on Node.left (280.0 open over 250ms QuadraticOut,
600.0 closed over 200ms QuadraticIn).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 7: Block 3 — Easing function gallery

**Files:**
- Modify: `cross-platform/bevy-ui-showcase/src/animations_section.rs`
- Modify: `cross-platform/bevy-ui-showcase/src/main.rs` (register one new system)

Six labeled rows, each with a small marker that races across a bar when "Restart" is clicked. Each marker uses a different `EaseFunction` so the curve shapes are visually comparable.

- [ ] **Step 7.1: Add components and the restart system in `animations_section.rs`.**

After the existing `toggle_drawer` system, add:

```rust
/// Marker on each easing-gallery marker entity, holding the EaseFunction
/// it should animate with.
#[derive(Component)]
pub(crate) struct EasingMarker(pub EaseFunction);

/// Marker on the "Restart" button.
#[derive(Component)]
pub(crate) struct RestartGallery;

/// On Pressed of RestartGallery, insert a fresh `Tween<Val>` on every
/// `EasingMarker` entity. All six animate in lockstep so the curves can
/// be compared side by side.
#[allow(clippy::type_complexity)]
pub fn restart_easing_gallery(
    mut commands: Commands,
    buttons: Query<&Interaction, (Changed<Interaction>, With<RestartGallery>)>,
    markers: Query<(Entity, &EasingMarker)>,
) {
    let mut clicked = false;
    for i in &buttons {
        if *i == Interaction::Pressed {
            clicked = true;
        }
    }
    if !clicked {
        return;
    }
    for (entity, marker) in &markers {
        commands.entity(entity).insert(Tween::<Val> {
            start: Val::Px(0.0),
            end: Val::Px(584.0),
            elapsed: 0.0,
            duration: 1.5,
            easing: marker.0,
        });
    }
}
```

- [ ] **Step 7.2: Add the Block 3 spawn inside the content closure (after Block 2).**

```rust
            // ── Block 3: Easing function gallery ──
            c.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|cell| {
                cell.spawn((
                    Text::new("Easing function gallery"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor::default(),
                    TextRole::Primary,
                ));
                cell.spawn((
                    Text::new(
                        "Six EaseFunction variants. Click Restart to race \
                         all six markers in lockstep over 1.5 seconds.",
                    ),
                    TextColor::default(),
                    TextRole::Subtle,
                ));

                let curves: [(&str, EaseFunction); 6] = [
                    ("Linear", EaseFunction::Linear),
                    ("QuadraticIn", EaseFunction::QuadraticIn),
                    ("QuadraticOut", EaseFunction::QuadraticOut),
                    ("QuadraticInOut", EaseFunction::QuadraticInOut),
                    ("ElasticOut", EaseFunction::ElasticOut),
                    ("BackOut", EaseFunction::BackOut),
                ];

                for (name, ease) in curves {
                    cell.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(12.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new(name),
                            TextFont { font_size: 12.0, ..default() },
                            TextColor::default(),
                            TextRole::Subtle,
                            Node {
                                width: Val::Px(120.0),
                                ..default()
                            },
                        ));
                        // Bar with absolutely-positioned marker inside.
                        row.spawn((
                            Node {
                                width: Val::Px(600.0),
                                height: Val::Px(32.0),
                                position_type: PositionType::Relative,
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor::default(),
                            BgRole::Surface,
                            BorderColor::default(),
                            BorderRole::Subtle,
                        ))
                        .with_children(|bar| {
                            bar.spawn((
                                Node {
                                    width: Val::Px(12.0),
                                    height: Val::Px(12.0),
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(0.0),
                                    top: Val::Px(10.0),
                                    border_radius: BorderRadius::all(Val::Px(9999.0)),
                                    ..default()
                                },
                                BackgroundColor::default(),
                                BgRole::BoxFill,
                                EasingMarker(ease),
                            ));
                        });
                    });
                }

                // Restart button below the gallery.
                cell.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor::default(),
                    BgRole::ButtonIdle,
                    RestartGallery,
                ))
                .with_child((
                    Text::new("Restart"),
                    TextColor::default(),
                    TextRole::Primary,
                ));
            });
```

- [ ] **Step 7.3: Register `animations_section::restart_easing_gallery` in `main.rs`.**

- [ ] **Step 7.4: Verify.**

```powershell
cargo check --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml
cargo clippy --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml -- -D warnings
```

Both exit 0.

- [ ] **Step 7.5: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/animations_section.rs cross-platform/bevy-ui-showcase/src/main.rs
git commit -m "feat(bevy-ui-showcase): easing gallery (Block 3)

Six EaseFunction variants race in lockstep over 1.5 seconds. Markers are
absolutely positioned inside a 600x32 bar, tweened via Tween<Val> on
Node.left from 0px to 584px.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 8: Block 4 — Color crossfade

**Files:**
- Modify: `cross-platform/bevy-ui-showcase/src/animations_section.rs`
- Modify: `cross-platform/bevy-ui-showcase/src/main.rs` (register one new system)

Two swatches side by side with a Toggle button. Each click crossfades both between two color pairs over 600ms.

- [ ] **Step 8.1: Add components and the toggle system in `animations_section.rs`.**

After `restart_easing_gallery`, add:

```rust
use crate::theme::Theme;

#[derive(Clone, Copy)]
pub(crate) enum CrossfadeSide {
    Left,
    Right,
}

#[derive(Component)]
pub(crate) struct CrossfadeSwatch(pub CrossfadeSide);

/// Lives on the toggle button. Flipped on each click; chooses which color
/// pair the swatches are tweened toward.
#[derive(Component, Default)]
pub(crate) struct CrossfadeState(pub bool);

/// On Pressed of a CrossfadeState button, flip the state and insert
/// `Tween<Color>` on each CrossfadeSwatch with the new target color.
///
/// Note: the swatches do not carry BgRole — color is owned by the most
/// recent toggle. A global theme toggle does NOT reskin the swatches;
/// they stay at whatever the prior click painted them in the prior theme.
/// The next click re-reads `theme.bg.*` and tweens to current theme's
/// pair colors.
#[allow(clippy::type_complexity)]
pub fn toggle_crossfade(
    mut commands: Commands,
    theme: Res<Theme>,
    mut state_q: Query<(&Interaction, &mut CrossfadeState), Changed<Interaction>>,
    swatches: Query<(Entity, &CrossfadeSwatch, &BackgroundColor)>,
) {
    for (interaction, mut state) in &mut state_q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        state.0 = !state.0;
        for (entity, side, current) in &swatches {
            let target = match (state.0, side.0) {
                (false, CrossfadeSide::Left) => theme.bg.box_fill,
                (false, CrossfadeSide::Right) => theme.bg.accent,
                (true, CrossfadeSide::Left) => theme.bg.slider_thumb,
                (true, CrossfadeSide::Right) => theme.bg.button_pressed,
            };
            commands.entity(entity).insert(Tween::<Color> {
                start: current.0,
                end: target,
                elapsed: 0.0,
                duration: 0.6,
                easing: EaseFunction::QuadraticInOut,
            });
        }
    }
}
```

- [ ] **Step 8.2: Add the Block 4 spawn inside the content closure (after Block 3).**

Pre-compute the initial pair-A colors outside the closure since we need them to seed the swatches at spawn time. Use `Theme::dark()` as the seed source (matches the app's default):

```rust
            // ── Block 4: Color crossfade ──
            let seed = crate::theme::Theme::dark();
            let seed_left = seed.bg.box_fill;
            let seed_right = seed.bg.accent;
            c.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|cell| {
                cell.spawn((
                    Text::new("Color crossfade"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor::default(),
                    TextRole::Primary,
                ));
                cell.spawn((
                    Text::new(
                        "Two swatches crossfade between two theme-derived \
                         color pairs over 600ms (QuadraticInOut). \
                         Swatches don't carry BgRole; theme toggle won't \
                         reskin them until the next Toggle click.",
                    ),
                    TextColor::default(),
                    TextRole::Subtle,
                ));
                cell.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Node {
                            width: Val::Px(80.0),
                            height: Val::Px(80.0),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(seed_left),
                        CrossfadeSwatch(CrossfadeSide::Left),
                    ));
                    row.spawn((
                        Node {
                            width: Val::Px(80.0),
                            height: Val::Px(80.0),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(seed_right),
                        CrossfadeSwatch(CrossfadeSide::Right),
                    ));
                });

                // Toggle button below the swatches.
                cell.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor::default(),
                    BgRole::ButtonIdle,
                    CrossfadeState::default(),
                ))
                .with_child((
                    Text::new("Toggle"),
                    TextColor::default(),
                    TextRole::Primary,
                ));
            });
```

- [ ] **Step 8.3: Register `animations_section::toggle_crossfade` in `main.rs`.**

- [ ] **Step 8.4: Verify.**

```powershell
cargo check --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml
cargo clippy --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml -- -D warnings
```

Both exit 0. `Tween<Color>` unused-warning from Task 1 should clear.

- [ ] **Step 8.5: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/animations_section.rs cross-platform/bevy-ui-showcase/src/main.rs
git commit -m "feat(bevy-ui-showcase): color crossfade demo (Block 4)

Two 80x80 swatches seeded with bg.box_fill / bg.accent. The Toggle button
flips CrossfadeState and inserts Tween<Color> on both swatches with the
new target pair (pair B = bg.slider_thumb / bg.button_pressed). 600ms
QuadraticInOut. Swatches deliberately don't carry BgRole — color is owned
by the most recent toggle; theme toggle does not reskin them.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 9: Final validation sweep

**Files:** none (verification only)

- [ ] **Step 9.1: Clippy clean.**

```powershell
cargo clippy --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml -- -D warnings
```

Expected: exit 0, zero warnings. If any new system has a complex Query that trips `clippy::type_complexity`, add `#[allow(clippy::type_complexity)]` to that specific system (per-function only, never module-wide).

- [ ] **Step 9.2: No leaked Color::srgb outside theme.rs.**

```powershell
Select-String -Path "cross-platform/bevy-ui-showcase/src/*.rs" -Pattern "Color::srgb" | Where-Object { $_.Path -notlike "*theme.rs" }
```

Expected: zero matches.

- [ ] **Step 9.3: Cargo run launches the app.**

```powershell
cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml
```

This is for the user to verify visually. Static checks pass; behavior is:

1. App launches in dark mode.
2. Hover any click-counter button (Layout tab "Click me" or Animations tab "Hover me") — visible scale-up, smooth.
3. Click theme toggle in the tab bar — colors blend over ~300ms (no snap).
4. Open Animations tab — four blocks render: Hover scale-up (single button), Slide-in panel (drawer demo), Easing gallery (six markers + Restart), Color crossfade (two swatches + Toggle).
5. Click "Toggle drawer" — panel slides in from the right; click again — slides out.
6. Click "Restart" in the easing gallery — six markers race the bar; visible curve differences (Linear is steady; QuadraticIn lags then catches up; QuadraticOut surges then settles; QuadraticInOut sigmoid; ElasticOut overshoots and settles; BackOut anticipates back, overshoots forward).
7. Click "Toggle" in color crossfade — both swatches blend smoothly between pair A (blue/purple) and pair B (red/light-gray).
8. Toggle theme mid-drawer-open or mid-crossfade — both animations continue without freezing.

- [ ] **Step 9.4: Update Phase 5 task status in the task tracker.**

Mark Phase 5 complete.

- [ ] **Step 9.5: Commit any final fixups from clippy / visual sweep.**

If steps 9.1 / 9.2 / 9.3 surfaced anything to fix, commit it as `fix(bevy-ui-showcase): post-Phase-5 cleanup`. Otherwise no commit needed.

---

## Out-of-scope reminder

The spec deliberately defers: tab-switch animations, hover color tween, tween cancellation on theme toggle, `Val::Percent` interpolation, sequencing/chaining/repeats, spring physics. If any of these come up during implementation, file a follow-up rather than scope-creeping this plan.
