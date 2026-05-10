# bevy-ui-showcase — Phase 5: Animations & transitions

**Date:** 2026-05-10
**Crate:** `cross-platform/bevy-ui-showcase`
**Issue:** [#7](https://github.com/mule/rust-doodle/issues/7) Phase 5
**Branch:** `7-bevy-ui-showcase-gallery-of-bevy_ui-features`
**Predecessor:** Phase 4 (Theming) shipped at `e5d1ed7`. This phase builds on the `Theme` resource, `BgRole`/`TextRole`/`BorderRole` components, and the role resolver systems introduced there.

## Goal

Demonstrate Bevy 0.18's animation patterns for UI: a hand-rolled generic
`Tween<T>` component driven by `bevy_math::EaseFunction` curves, applied both
locally (a new Animations tab with four discrete demos) and selectively across
the existing app (theme toggle crossfade, hover scale-up on counter buttons).

## Scope

**Two cross-cutting integrations + four in-tab demos.** No third-party deps.

Out of scope:
- Tab-switch transitions (would require reworking section visibility from
  `display: None ↔ Flex` to absolute-positioning; deferred).
- Universal hover *color* tween (kept instant; only scale animates).
- Tween cancellation when theme toggles mid-tween (benign snap; accepted).
- `Val::Percent` interpolation (Px-only).
- Sequencing / chaining / repeating tweens. Each tween is one-shot.

## Architecture

### The `Tween<T>` primitive

```rust
// src/tween.rs

use bevy::math::curve::EaseFunction;

#[derive(Component)]
pub struct Tween<T: Clone + Send + Sync + 'static> {
    pub start: T,
    pub end: T,
    pub elapsed: f32,
    pub duration: f32,
    pub easing: EaseFunction,
}
```

Generic component. Each concrete `T` is its own ECS-distinct component type.
Three concrete types are used:

| `T`     | Writes to                | Demos                      |
|---------|--------------------------|----------------------------|
| `f32`   | `UiTransform.scale`      | hover scale-up             |
| `Color` | `BackgroundColor`        | crossfade demo             |
| `Val`   | `Node.right`             | slide-in panel             |

`Val` is restricted to `Val::Px(_)` for both `start` and `end`. The advance
system asserts in debug builds; in release, mixed kinds blend by treating
non-Px values as `Val::Px(0.0)` (a documented gotcha — every callsite passes
`Val::Px`).

### Three advance systems, one per concrete `T`

Each system follows the same shape:

```rust
pub fn advance_f32_tweens(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut Tween<f32>, &mut UiTransform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tween, mut transform) in &mut q {
        tween.elapsed += dt;
        let t = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
        let eased = tween.easing.sample(t).unwrap_or(t);
        let value = tween.start.lerp(tween.end, eased);
        transform.scale = Vec2::splat(value);
        if t >= 1.0 {
            commands.entity(entity).remove::<Tween<f32>>();
        }
    }
}
```

`advance_color_tweens` substitutes `Color::mix` and writes `BackgroundColor.0`.
`advance_val_tweens` substitutes `lerp_val_px` (extracts the Px value, lerps,
wraps back in `Val::Px`) and writes `Node.right`.

`EaseFunction::sample(t)` returns `Option<f32>` because some curves are
piecewise; the default `unwrap_or(t)` falls back to linear if a curve fails to
sample (shouldn't happen for the curves we use, but the API requires it).

Bevy's tuple-of-systems limit (20) is comfortable: we add 4 advance systems
across both this phase and (in retrospect) Phase 4. The split into two
`add_systems(Update, ...)` calls established in Phase 4 stays.

### Lifecycle

1. Some interaction or click handler inserts a `Tween<T>` on an entity.
2. Each frame, the advance system increments `elapsed`, computes eased
   progress, interpolates, writes the target component.
3. When `elapsed >= duration`, the advance system queues
   `commands.entity(e).remove::<Tween<T>>()`. The tween is gone next frame.
4. If a new tween of the same `T` is inserted on an entity that already has
   one, Bevy's component overwrite semantics replace it. The new tween's
   `start` value is whatever the caller passed, so callers always read the
   *current* property value (e.g. `transform.scale.x`) when constructing the
   replacement.

## Cross-cutting integrations

### Integration A: Theme toggle crossfade

Currently `handle_theme_toggle` calls `theme.toggle()`, and the role
resolvers re-run that frame, snapping every BgRole / TextRole / BorderRole
node to its new color. Replace the snap with a 300ms blend.

```rust
// src/theme.rs additions

#[derive(Resource)]
pub struct ThemeTransition {
    pub from_bg: BgTokens,
    pub from_text: TextTokens,
    pub from_border: BorderTokens,
    pub elapsed: f32,
    pub duration: f32,        // 0.3
    pub easing: EaseFunction, // QuadInOut
}

impl BgTokens {
    pub fn mix(a: &BgTokens, b: &BgTokens, t: f32) -> BgTokens {
        BgTokens {
            background: a.background.mix(&b.background, t),
            surface: a.surface.mix(&b.surface, t),
            // ... per field
        }
    }
}
// Similar mix impls on TextTokens, BorderTokens.
```

`handle_theme_toggle` now:

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
                from_bg, from_text, from_border,
                elapsed: 0.0,
                duration: 0.3,
                easing: EaseFunction::QuadraticInOut,
            });
        }
    }
}
```

The `transition.is_none()` guard ignores rapid double-toggles while a fade is
already in progress. Single-toggle UX, but stable.

A new `advance_theme_transition` system in `theme.rs`:

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

The three role resolvers gain one branch:

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
        let progress = (t.elapsed / t.duration).clamp(0.0, 1.0);
        let eased = t.easing.sample(progress).unwrap_or(progress);
        BgTokens::mix(&t.from_bg, &theme.bg, eased)
    } else {
        theme.bg
    };
    for (role, mut bg) in &mut q {
        bg.0 = blended_bg.resolve(*role);
    }
}
```

Same change to `resolve_text_role` and `resolve_border_role`. `sync_clear_color`
also uses the blended background during transition (otherwise the window's
clear color would snap while the UI surfaces crossfade — visible glitch).

### Integration B: Hover scale-tween on `ClickCount` buttons

Add a parallel system to `update_button_visuals` (which keeps doing its
existing instant color writes). The new system, `dispatch_button_hover_scale`,
inserts a `Tween<f32>` on `Changed<Interaction>` for entities with
`ClickCount`:

```rust
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

Reading `transform.scale.x` mid-tween means a new hover during an in-flight
hover-out cleanly continues from the visible scale rather than snapping to
1.0. No tween-cancellation logic needed.

`UiTransform` is auto-inserted by Bevy 0.18 on every `Node` entity, so we
don't need to spawn it manually on the click-counter buttons.

## Animations tab content

Layout mirrors the Theming tab — header + scrollable content column with
labeled blocks, separated by spacing.

### Block 1: Hover scale-up

```
Hover scale-up
  ┌──────────────┐
  │  Hover me    │
  └──────────────┘
  Scales from 1.0 to 1.05 over 150ms (QuadOut). Color stays instant — see
  the architecture note in the source.
```

A single `ClickCount`-marked button. The global `dispatch_button_hover_scale`
applies automatically. ~10 lines of spawn code. The block doubles as a
canonical "this is what hover scale looks like" reference.

### Block 2: Slide-in panel

```
Slide-in panel
  ┌─────────────────────────────────────────────────┐
  │  (demo container, 600×300, overflow: clip)       │
  │                                              ┌───┤
  │                                              │ D │
  │                                              │ r │  ← panel slides
  │                                              │ a │     in from the
  │                                              │ w │     right
  │                                              │ e │
  │                                              │ r │
  │                                              └───┤
  └─────────────────────────────────────────────────┘
  [ Toggle drawer ]
```

The panel:
- `Node { position_type: PositionType::Absolute, top: 0, bottom: 0, right: Val::Px(-340.0), width: Val::Px(320.0), .. }`
- Initial `right: Val::Px(-340.0)` — fully off-screen (320 width + 20 buffer).
- `BgRole::Surface` for the body, `BorderRole::Subtle` for a 1px left edge.

Components on the panel: `DrawerPanel` marker + `DrawerOpen(bool)` (initially `false`).

The toggle button is a `ClickCount`-free entity (we don't want hover scale on
this one — it's a UI control, not the demo subject). On click, a new system
`toggle_drawer` runs: flip the bool, insert `Tween<Val>` on the panel.

```rust
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
    if !clicked { return; }
    for (entity, node, mut open) in &mut panels {
        open.0 = !open.0;
        let current = match node.right { Val::Px(v) => v, _ => 0.0 };
        let (target, duration, easing) = if open.0 {
            (0.0, 0.25, EaseFunction::QuadraticOut)
        } else {
            (-340.0, 0.20, EaseFunction::QuadraticIn)
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

The demo container has `overflow: Overflow::clip()` so the off-screen part of
the panel doesn't bleed into the rest of the layout.

### Block 3: Easing function gallery

Six rows, each:
- 600×32 background bar (`BgRole::Surface`), `BorderRole::Subtle`, `border_radius: sm`.
- Inside, a 12×12 marker circle (`BgRole::BoxFill`, `border_radius: pill`),
  `Node { position_type: Absolute, left: Val::Px(0.0), top: Val::Px(10.0), .. }`.
- A 100px-wide label to the left of the bar showing the easing name.

The marker carries `EasingMarker(EaseFunction)`. The six are:

| Variant                   | Visual character             |
|---------------------------|------------------------------|
| `Linear`                  | constant speed               |
| `QuadraticIn`             | starts slow, accelerates     |
| `QuadraticOut`            | starts fast, decelerates     |
| `QuadraticInOut`          | sigmoid (ease-in-out)        |
| `ElasticOut`              | overshoots, oscillates, settles |
| `BackOut`                 | anticipates back, overshoots forward |

A "Restart" button below the gallery runs:

```rust
pub fn restart_easing_gallery(
    mut commands: Commands,
    buttons: Query<&Interaction, (Changed<Interaction>, With<RestartGallery>)>,
    markers: Query<(Entity, &EasingMarker)>,
) {
    let mut clicked = false;
    for i in &buttons { if *i == Interaction::Pressed { clicked = true; } }
    if !clicked { return; }
    for (entity, marker) in &markers {
        commands.entity(entity).insert(Tween::<Val> {
            start: Val::Px(0.0),
            end: Val::Px(584.0),  // 600 bar width − 12 marker − 4 right pad
            elapsed: 0.0,
            duration: 1.5,
            easing: marker.0,
        });
    }
}
```

All six markers tween in lockstep so the curve shapes are visually
comparable.

### Block 4: Color crossfade

Two 80×80 swatches side by side with a Toggle button below. Each click
crossfades both swatches between two color pairs over 600ms, `QuadInOut`.

```rust
#[derive(Component)]
pub struct CrossfadeState(pub bool);

#[derive(Component)]
pub struct CrossfadeSwatch(pub CrossfadeSide);

pub enum CrossfadeSide { Left, Right }
```

Pair A (state false): left = `theme.bg.box_fill`, right = `theme.bg.accent`.
Pair B (state true):  left = `theme.bg.slider_thumb`, right = `theme.bg.button_pressed`.

```rust
pub fn toggle_crossfade(
    mut commands: Commands,
    theme: Res<Theme>,
    mut state_q: Query<(&Interaction, &mut CrossfadeState), Changed<Interaction>>,
    swatches: Query<(Entity, &CrossfadeSwatch, &BackgroundColor)>,
) {
    for (interaction, mut state) in &mut state_q {
        if *interaction != Interaction::Pressed { continue; }
        state.0 = !state.0;
        for (entity, side, current) in &swatches {
            let target = match (state.0, side) {
                (false, CrossfadeSwatch::Left) => theme.bg.box_fill,
                (false, CrossfadeSwatch::Right) => theme.bg.accent,
                (true,  CrossfadeSwatch::Left) => theme.bg.slider_thumb,
                (true,  CrossfadeSwatch::Right) => theme.bg.button_pressed,
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

The swatches deliberately use existing theme tokens, but they do **not**
carry `BgRole` components — color is owned entirely by the `Tween<Color>` /
the most recent `toggle_crossfade` write. Consequence: a theme toggle does
not automatically reskin the crossfade swatches. They stay at whatever color
the last toggle painted them in the *prior* theme until the next click of
the Toggle button. Documented as a code comment; the demo still works
because clicking Toggle re-reads `theme.bg.*` and tweens to the current
theme's pair colors.

## File structure

**New:**
- `src/tween.rs` — `Tween<T>` generic, three advance systems, `lerp_val_px`
  helper.
- `src/animations_section.rs` — Animations tab content. Components:
  `EasingMarker`, `RestartGallery`, `DrawerPanel`, `DrawerOpen`,
  `DrawerToggle`, `CrossfadeState`, `CrossfadeSwatch`. Systems:
  `toggle_drawer`, `restart_easing_gallery`, `toggle_crossfade`.

**Modified:**
- `src/theme.rs` — `ThemeTransition` resource, `BgTokens::mix` /
  `TextTokens::mix` / `BorderTokens::mix` impls,
  `advance_theme_transition` system, the three resolver systems
  blend during transition, `handle_theme_toggle` snapshots before
  toggling and inserts the transition resource, `sync_clear_color`
  reads blended background during transition.
- `src/widgets_section.rs` — new `dispatch_button_hover_scale` system. The
  existing `update_button_visuals` is unchanged (color stays instant).
- `src/main.rs` — register tween advance systems, `advance_theme_transition`,
  the three Animations-tab interaction systems
  (`toggle_drawer`, `restart_easing_gallery`, `toggle_crossfade`),
  `dispatch_button_hover_scale`, and replace the
  `Section::Animations` placeholder with `animations_section::spawn`.

**No new dependencies.** `Cargo.toml` stays at `bevy = "0.18.1"`.

## Validation

- [ ] `cargo build --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
- [ ] `cargo clippy --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml -- -D warnings`
- [ ] `cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
- [ ] Hover any click-counter button — visible scale-up, smooth.
- [ ] Click theme toggle — colors crossfade over ~300ms (no snap).
- [ ] Animations tab opens — all four blocks render.
- [ ] Hover the Animations-tab "Hover me" button — scales up.
- [ ] Click "Toggle drawer" — panel slides in from the right; click again — slides out.
- [ ] Click "Restart" in the easing gallery — six markers race the bar; visible curve differences.
- [ ] Click "Toggle" in color crossfade — swatches blend smoothly between the two pairs.
- [ ] Theme toggle during a drawer-open or crossfade-in-progress — both animations continue without freezing or snapping.

## Out of scope / follow-ups

- Tab-switch animations.
- Hover color tween on buttons.
- Tween cancellation on theme toggle (accepted snap).
- `Val::Percent` interpolation.
- Sequencing / chaining (e.g. "after slide-in, fade in panel content").
- Repeats / yoyo / loops.
- Spring physics (would need a proper damped-harmonic-oscillator integrator,
  not just `EaseFunction`).
- Animated theme transitions for `RadiusTokens` (radii don't change between
  modes, so nothing to animate).
