# bevy-ui-showcase — Phase 4: Theming Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate the bevy-ui-showcase app from hardcoded `Color::srgb(...)` constants to a centralized `Theme` resource with dark/light variants, a tab-bar toggle that reskins all four sections live, and a Theming tab that demonstrates the system (palette, border radii, border colors, typography with a custom font).

**Architecture:** A `Theme` Resource holds nested token structs (`BgTokens`, `TextTokens`, `BorderTokens`, `RadiusTokens`). Themed nodes carry small role enum components (`BgRole`, `TextRole`, `BorderRole`); three resolver systems write the resolved color whenever the theme changes or a role is newly added. Widget interaction systems (hover/pressed/active state writes) gain a `theme: Res<Theme>` parameter and read live from the resource. A `ClearColor` sync system mirrors the background token to the window clear color.

**Tech Stack:** Bevy 0.18.1, no test harness (Bevy UI verification is `cargo run` + visual inspection). Spec: `docs/superpowers/specs/2026-05-09-bevy-ui-showcase-theming-design.md`.

---

## Notes for the executor

- This crate has **no automated test suite**. "Verify" steps are either `cargo check` (compile correctness) or `cargo run` (visual confirmation). Where a step says "verify visually," the user will be running the app and looking at it.
- Bevy 0.18.1-specific gotchas already encountered in this codebase: `Event` → `Message`, `EventReader` → `MessageReader`, `parent_entity()` → `target_entity()`, UI uses `UiTransform`/`UiGlobalTransform` (not `Transform`/`GlobalTransform`).
- Build command: `cargo check --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`. Run command: `cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`.
- Commit cadence: one commit per task. Use the conventional-commit style already in this repo (`feat(bevy-ui-showcase): …`, `refactor(bevy-ui-showcase): …`).

---

## Task 1: Create `theme.rs` — full module

**Files:**
- Create: `cross-platform/bevy-ui-showcase/src/theme.rs`
- Modify: `cross-platform/bevy-ui-showcase/src/main.rs` (add `mod theme;`, register `Theme` resource, register all systems from theme.rs)

This task ships the entire theming infrastructure as one self-contained module: token types, role enum components, dark/light constructors, three role-resolver systems, clear-color sync, and the toggle handler. Nothing in the rest of the app calls into it yet — everything still uses the old `Color::srgb(...)` constants. That's intentional: this task must compile cleanly in isolation so subsequent migration tasks can swap call sites one file at a time.

- [ ] **Step 1.1: Create `src/theme.rs` with the full module.**

```rust
use bevy::prelude::*;

// ── Theme mode ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

// ── Token structs ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub struct BgTokens {
    pub background: Color,
    pub surface: Color,
    pub box_fill: Color,
    pub accent: Color,
    pub button_idle: Color,
    pub button_hover: Color,
    pub button_pressed: Color,
    pub tab_bar: Color,
    pub tab_inactive: Color,
    pub tab_hovered: Color,
    pub tab_active: Color,
    pub input: Color,
    pub emoji_btn_idle: Color,
    pub emoji_btn_hover: Color,
}

#[derive(Clone, Copy, Debug)]
pub struct TextTokens {
    pub primary: Color,
    pub subtle: Color,
    pub on_accent: Color,
}

#[derive(Clone, Copy, Debug)]
pub struct BorderTokens {
    pub subtle: Color,
    pub focus: Color,
}

#[derive(Clone, Copy, Debug)]
pub struct RadiusTokens {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub pill: f32,
}

// ── Theme resource ──────────────────────────────────────────────────────────

#[derive(Resource, Clone, Debug)]
pub struct Theme {
    pub mode: ThemeMode,
    pub bg: BgTokens,
    pub text: TextTokens,
    pub border: BorderTokens,
    pub radius: RadiusTokens,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            bg: BgTokens {
                background: Color::srgb(0.08, 0.09, 0.12),
                surface: Color::srgb(0.13, 0.14, 0.17),
                box_fill: Color::srgb(0.20, 0.40, 0.60),
                accent: Color::srgb(0.50, 0.30, 0.70),
                button_idle: Color::srgb(0.22, 0.24, 0.30),
                button_hover: Color::srgb(0.30, 0.34, 0.42),
                button_pressed: Color::srgb(0.40, 0.46, 0.56),
                tab_bar: Color::srgb(0.10, 0.11, 0.14),
                tab_inactive: Color::srgb(0.18, 0.19, 0.22),
                tab_hovered: Color::srgb(0.24, 0.25, 0.28),
                tab_active: Color::srgb(0.30, 0.32, 0.40),
                input: Color::srgb(0.10, 0.11, 0.14),
                emoji_btn_idle: Color::srgb(0.20, 0.22, 0.28),
                emoji_btn_hover: Color::srgb(0.30, 0.34, 0.40),
            },
            text: TextTokens {
                primary: Color::srgb(0.92, 0.93, 0.95),
                subtle: Color::srgb(0.55, 0.57, 0.62),
                on_accent: Color::WHITE,
            },
            border: BorderTokens {
                subtle: Color::srgb(0.30, 0.32, 0.38),
                focus: Color::srgb(0.55, 0.65, 0.85),
            },
            radius: RadiusTokens { sm: 4.0, md: 8.0, lg: 16.0, pill: 9999.0 },
        }
    }

    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            bg: BgTokens {
                background: Color::srgb(0.96, 0.96, 0.97),
                surface: Color::srgb(0.91, 0.92, 0.94),
                box_fill: Color::srgb(0.30, 0.55, 0.85),
                accent: Color::srgb(0.65, 0.45, 0.85),
                button_idle: Color::srgb(0.85, 0.86, 0.90),
                button_hover: Color::srgb(0.78, 0.80, 0.85),
                button_pressed: Color::srgb(0.70, 0.73, 0.80),
                tab_bar: Color::srgb(0.88, 0.89, 0.91),
                tab_inactive: Color::srgb(0.82, 0.83, 0.86),
                tab_hovered: Color::srgb(0.74, 0.76, 0.80),
                tab_active: Color::srgb(0.65, 0.68, 0.78),
                input: Color::srgb(0.98, 0.98, 0.99),
                emoji_btn_idle: Color::srgb(0.84, 0.85, 0.89),
                emoji_btn_hover: Color::srgb(0.74, 0.76, 0.81),
            },
            text: TextTokens {
                primary: Color::srgb(0.12, 0.13, 0.16),
                subtle: Color::srgb(0.40, 0.42, 0.48),
                on_accent: Color::WHITE,
            },
            border: BorderTokens {
                subtle: Color::srgb(0.70, 0.72, 0.76),
                focus: Color::srgb(0.30, 0.45, 0.75),
            },
            radius: RadiusTokens { sm: 4.0, md: 8.0, lg: 16.0, pill: 9999.0 },
        }
    }

    pub fn toggle(&mut self) {
        *self = match self.mode {
            ThemeMode::Dark => Self::light(),
            ThemeMode::Light => Self::dark(),
        };
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

// ── Role enum components ────────────────────────────────────────────────────

#[derive(Component, Clone, Copy, Debug)]
pub enum BgRole {
    Background,
    Surface,
    BoxFill,
    Accent,
    ButtonIdle,
    TabBar,
    TabInactive,
    Input,
    EmojiBtnIdle,
}

#[derive(Component, Clone, Copy, Debug)]
pub enum TextRole {
    Primary,
    Subtle,
    OnAccent,
}

#[derive(Component, Clone, Copy, Debug)]
pub enum BorderRole {
    Subtle,
    Focus,
}

impl BgTokens {
    pub fn resolve(&self, role: BgRole) -> Color {
        match role {
            BgRole::Background => self.background,
            BgRole::Surface => self.surface,
            BgRole::BoxFill => self.box_fill,
            BgRole::Accent => self.accent,
            BgRole::ButtonIdle => self.button_idle,
            BgRole::TabBar => self.tab_bar,
            BgRole::TabInactive => self.tab_inactive,
            BgRole::Input => self.input,
            BgRole::EmojiBtnIdle => self.emoji_btn_idle,
        }
    }
}

impl TextTokens {
    pub fn resolve(&self, role: TextRole) -> Color {
        match role {
            TextRole::Primary => self.primary,
            TextRole::Subtle => self.subtle,
            TextRole::OnAccent => self.on_accent,
        }
    }
}

impl BorderTokens {
    pub fn resolve(&self, role: BorderRole) -> Color {
        match role {
            BorderRole::Subtle => self.subtle,
            BorderRole::Focus => self.focus,
        }
    }
}

// ── Marker for the toggle button ────────────────────────────────────────────

#[derive(Component)]
pub struct ThemeToggle;

// ── Resolver systems ────────────────────────────────────────────────────────
//
// Pattern: each resolver runs whenever the Theme resource was mutated this
// frame (full sweep) OR when a role component was newly added to an entity
// (incremental). The "added" probe is a read-only query of `Entity` — keeping
// the mutable color query as the only mutable access avoids ParamSet conflicts.

pub fn resolve_bg_role(
    theme: Res<Theme>,
    mut q: Query<(&BgRole, &mut BackgroundColor)>,
    added: Query<(), Added<BgRole>>,
) {
    if !theme.is_changed() && added.is_empty() {
        return;
    }
    for (role, mut bg) in &mut q {
        bg.0 = theme.bg.resolve(*role);
    }
}

pub fn resolve_text_role(
    theme: Res<Theme>,
    mut q: Query<(&TextRole, &mut TextColor)>,
    added: Query<(), Added<TextRole>>,
) {
    if !theme.is_changed() && added.is_empty() {
        return;
    }
    for (role, mut color) in &mut q {
        color.0 = theme.text.resolve(*role);
    }
}

pub fn resolve_border_role(
    theme: Res<Theme>,
    mut q: Query<(&BorderRole, &mut BorderColor)>,
    added: Query<(), Added<BorderRole>>,
) {
    if !theme.is_changed() && added.is_empty() {
        return;
    }
    for (role, mut color) in &mut q {
        color.0 = theme.border.resolve(*role);
    }
}

// ── Clear-color sync ────────────────────────────────────────────────────────

pub fn sync_clear_color(theme: Res<Theme>, mut clear: ResMut<ClearColor>) {
    if !theme.is_changed() {
        return;
    }
    clear.0 = theme.bg.background;
}

// ── Toggle handler ──────────────────────────────────────────────────────────

pub fn handle_theme_toggle(
    mut theme: ResMut<Theme>,
    q: Query<&Interaction, (Changed<Interaction>, With<ThemeToggle>)>,
) {
    for interaction in &q {
        if *interaction == Interaction::Pressed {
            theme.toggle();
        }
    }
}
```

- [ ] **Step 1.2: Wire into `main.rs`.**

Add `mod theme;` near the top alongside the other `mod` lines. Replace the `insert_resource(ClearColor(...))` line with theme initialization, and register every theme system in `Update`:

```rust
// near the top of main.rs, alongside `mod nav;` etc.
mod theme;

// inside fn main(), DELETE the existing line:
//     .insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.12)))
// and replace with theme initialization. Bevy's DefaultPlugins inserts a
// ClearColor with its own default; sync_clear_color overwrites it on the
// first Update frame using the theme's background token, so we end up with
// a single source of truth (no Color::srgb literals in main.rs).
        .insert_resource(theme::Theme::default())
```

In the `Update` system tuple, append the theme systems. Bevy 0.18 allows tuples of up to 20 systems; we already have ~18, so we'll need to split into two `.add_systems(Update, (...))` calls. Replace the single `Update` block with two:

```rust
        .add_systems(
            Update,
            (
                nav::handle_tab_clicks,
                nav::update_section_visibility,
                nav::update_tab_visuals,
                layout_section::update_percent_labels,
                widgets_section::update_button_visuals,
                widgets_section::update_click_buttons,
                widgets_section::update_checkboxes,
                widgets_section::update_slider_drag,
                widgets_section::position_slider_thumbs,
                widgets_section::update_slider_value_labels,
                widgets_section::update_cursor_for_sliders,
                widgets_section::update_text_input_focus,
                widgets_section::update_text_input_keyboard,
                widgets_section::update_text_input_display,
                widgets_section::update_text_input_border,
                widgets_section::toggle_emoji_menu,
                widgets_section::handle_emoji_clicks,
                widgets_section::update_emoji_button_visuals,
            ),
        )
        .add_systems(
            Update,
            (
                theme::resolve_bg_role,
                theme::resolve_text_role,
                theme::resolve_border_role,
                theme::sync_clear_color,
                theme::handle_theme_toggle,
            ),
        )
```

- [ ] **Step 1.3: Verify it compiles.**

Run: `cargo check --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
Expected: no errors, possibly `unused` warnings on some token fields and the `ThemeToggle` marker — that's fine, they get used in later tasks.

- [ ] **Step 1.4: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/theme.rs cross-platform/bevy-ui-showcase/src/main.rs
git commit -m "feat(bevy-ui-showcase): theming module — Theme resource, role components, resolvers"
```

---

## Task 2: Migrate `nav.rs` to theme tokens

**Files:**
- Modify: `cross-platform/bevy-ui-showcase/src/nav.rs`

Existing constants in `nav.rs` (`TAB_BAR_BG`, `TAB_INACTIVE`, `TAB_HOVERED`, `TAB_ACTIVE`, `TAB_TEXT`) get replaced. Tab buttons get `BgRole::TabInactive` + `TextRole::Primary`. Tab bar gets `BgRole::TabBar`. The `update_tab_visuals` system gains `theme: Res<Theme>` and reads `tab_active` / `tab_hovered` / `tab_inactive` from it.

- [ ] **Step 2.1: Replace the constants block.**

In `nav.rs`, delete the five `const` lines (currently `TAB_BAR_BG`, `TAB_INACTIVE`, `TAB_HOVERED`, `TAB_ACTIVE`, `TAB_TEXT`) and add a `use` line:

```rust
use crate::theme::{BgRole, TextRole, Theme};
```

- [ ] **Step 2.2: Update `spawn_tab_bar` to attach role components instead of literal colors.**

```rust
pub fn spawn_tab_bar(commands: &mut Commands) -> Entity {
    let bar = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(48.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor::default(),
            BgRole::TabBar,
        ))
        .id();

    let mut buttons = Vec::with_capacity(Section::ALL.len());
    for section in Section::ALL {
        let btn = commands
            .spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor::default(),
                BgRole::TabInactive,
                TabButton(section),
            ))
            .with_child((
                Text::new(section.label()),
                TextColor::default(),
                TextRole::Primary,
            ))
            .id();
        buttons.push(btn);
    }
    commands.entity(bar).add_children(&buttons);
    bar
}
```

- [ ] **Step 2.3: Update `update_tab_visuals` to read from `Theme`.**

```rust
pub fn update_tab_visuals(
    theme: Res<Theme>,
    current: Res<CurrentSection>,
    mut query: Query<(&Interaction, &TabButton, &mut BackgroundColor)>,
) {
    for (interaction, tab, mut bg) in &mut query {
        let is_active = tab.0 == current.0;
        bg.0 = match (is_active, *interaction) {
            (true, _) => theme.bg.tab_active,
            (false, Interaction::Hovered | Interaction::Pressed) => theme.bg.tab_hovered,
            (false, Interaction::None) => theme.bg.tab_inactive,
        };
    }
}
```

- [ ] **Step 2.4: Verify visually.**

Run: `cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
Expected: app launches, tab bar looks identical to before — same dark background, same inactive/hovered/active states. No visual change is the success criterion.

- [ ] **Step 2.5: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/nav.rs
git commit -m "refactor(bevy-ui-showcase): migrate nav.rs to theme tokens"
```

---

## Task 3: Migrate `layout_section.rs` to theme tokens

**Files:**
- Modify: `cross-platform/bevy-ui-showcase/src/layout_section.rs`

Replace `TEXT_COLOR`, `SUBTLE_COLOR`, `DEMO_BG`, `BOX_COLOR`, `ACCENT_COLOR` with role components. Every `Text` spawn gets `TextRole::Primary` or `TextRole::Subtle` (or `TextRole::OnAccent` for the white text on the accent strip in Demo 2). Every demo container gets `BgRole::Surface`. Demo boxes get `BgRole::BoxFill` or `BgRole::Accent`.

- [ ] **Step 3.1: Delete the `const` block at the top, add the import.**

```rust
use crate::theme::{BgRole, TextRole};
```

- [ ] **Step 3.2: Walk every spawn site and swap the literal color for the role component.**

Pattern: any `BackgroundColor(SOMETHING)` becomes `(BackgroundColor::default(), BgRole::X)`. Any `TextColor(SOMETHING)` becomes `(TextColor::default(), TextRole::X)`.

Specific mappings for `layout_section.rs`:
- `TextColor(TEXT_COLOR)` (3 occurrences — section header, demo titles) → `(TextColor::default(), TextRole::Primary)`
- `TextColor(SUBTLE_COLOR)` (2-3 occurrences — section subtitle, demo descriptions) → `(TextColor::default(), TextRole::Subtle)`
- `TextColor(Color::WHITE)` on the accent strip in Demo 2 → `(TextColor::default(), TextRole::OnAccent)`
- `BackgroundColor(DEMO_BG)` (3 occurrences — one per demo's row container) → `(BackgroundColor::default(), BgRole::Surface)`
- `BackgroundColor(BOX_COLOR)` → `(BackgroundColor::default(), BgRole::BoxFill)`
- `BackgroundColor(ACCENT_COLOR)` → `(BackgroundColor::default(), BgRole::Accent)`
- `TextColor(Color::WHITE)` on the percent-box labels in Demo 3 → `(TextColor::default(), TextRole::OnAccent)` (white on the colored boxes)

A typical change at a spawn site looks like:

```rust
// before
header.spawn((
    Text::new("Layout & Flexbox"),
    TextFont { font_size: 32.0, ..default() },
    TextColor(TEXT_COLOR),
));

// after
header.spawn((
    Text::new("Layout & Flexbox"),
    TextFont { font_size: 32.0, ..default() },
    TextColor::default(),
    TextRole::Primary,
));
```

And for a Node with `BackgroundColor`:

```rust
// before
cell.spawn((
    Node { /* … */ },
    BackgroundColor(DEMO_BG),
))

// after
cell.spawn((
    Node { /* … */ },
    BackgroundColor::default(),
    BgRole::Surface,
))
```

- [ ] **Step 3.3: Verify visually.**

Run: `cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
Expected: Layout tab looks identical — same headers, same demo containers, same blue/purple boxes. Pay attention to: header/subtitle contrast (text colors), the three demo container backgrounds (surface), and the percent-box live labels (still update each frame).

- [ ] **Step 3.4: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/layout_section.rs
git commit -m "refactor(bevy-ui-showcase): migrate layout_section.rs to theme tokens"
```

---

## Task 4: Migrate `widgets_section.rs` to theme tokens

**Files:**
- Modify: `cross-platform/bevy-ui-showcase/src/widgets_section.rs`

This is the biggest migration: ~13 color constants and 4 widget interaction systems that need a `theme: Res<Theme>` parameter. Map by the table from the spec. Key insight for this task: **role components hold the IDLE state; widget interaction systems read theme directly for hover/pressed/active.**

- [ ] **Step 4.1: Delete every color const at the top, add the import.**

The 9 deletions (line numbers from current file): `TEXT_COLOR`, `SUBTLE_COLOR`, `DEMO_BG`, `BTN_IDLE`, `BTN_HOVER`, `BTN_PRESSED`, `INPUT_BORDER_IDLE`, `INPUT_BORDER_FOCUSED`, `INPUT_BG`, `EMOJI_BTN_IDLE`, `EMOJI_BTN_HOVER`, `CHECKBOX_BORDER`, `CHECKBOX_BG`.

```rust
use crate::theme::{BgRole, BorderRole, TextRole, Theme};
```

- [ ] **Step 4.2: Migrate spawn-site colors to role components.**

Mapping:
- `TEXT_COLOR` → `TextRole::Primary`
- `SUBTLE_COLOR` → `TextRole::Subtle`
- `DEMO_BG` → `BgRole::Surface`
- `BTN_IDLE` → spawn site uses `BgRole::ButtonIdle`
- `INPUT_BG` and `CHECKBOX_BG` (same value today) → both `BgRole::Input`
- `INPUT_BORDER_IDLE` and `CHECKBOX_BORDER` → both `BorderRole::Subtle` (note: this unifies them — current `CHECKBOX_BORDER` is brighter than `INPUT_BORDER_IDLE`; after migration both use the input value, which is a small intentional design unification)
- `EMOJI_BTN_IDLE` → `BgRole::EmojiBtnIdle`
- `Color::WHITE` text on click counter / on emoji glyphs → `TextRole::OnAccent` if the surface behind is colored, else `TextRole::Primary` if behind is just neutral surface — eyeball each case during the pass

Same spawn-site pattern as Task 3: `BackgroundColor(X)` → `(BackgroundColor::default(), BgRole::X)`, `TextColor(X)` → `(TextColor::default(), TextRole::X)`, `BorderColor(X)` → `(BorderColor::default(), BorderRole::X)`.

- [ ] **Step 4.3: Update `update_button_visuals` to read theme.**

```rust
pub fn update_button_visuals(
    theme: Res<Theme>,
    mut q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<ClickCount>)>,
) {
    for (interaction, mut bg) in &mut q {
        bg.0 = match *interaction {
            Interaction::Pressed => theme.bg.button_pressed,
            Interaction::Hovered => theme.bg.button_hover,
            Interaction::None => theme.bg.button_idle,
        };
    }
}
```

(If `update_button_visuals` currently has a different signature in the file, preserve the filter set but add the `theme: Res<Theme>` first parameter and replace the literals with `theme.bg.…`.)

- [ ] **Step 4.4: Update `update_text_input_border` to read theme.**

```rust
pub fn update_text_input_border(
    theme: Res<Theme>,
    focused: Res<FocusedTextInput>,
    mut q: Query<(Entity, &mut BorderColor), With<TextInput>>,
) {
    for (entity, mut border) in &mut q {
        border.0 = if focused.0 == Some(entity) {
            theme.border.focus
        } else {
            theme.border.subtle
        };
    }
}
```

(Adjust the system signature to match what's actually in the file — the goal is replace literal `INPUT_BORDER_FOCUSED` / `INPUT_BORDER_IDLE` with `theme.border.focus` / `theme.border.subtle`.)

- [ ] **Step 4.5: Update `update_emoji_button_visuals` to read theme.**

```rust
pub fn update_emoji_button_visuals(
    theme: Res<Theme>,
    mut q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<EmojiMenuButton>)>,
) {
    for (interaction, mut bg) in &mut q {
        bg.0 = match *interaction {
            Interaction::Hovered | Interaction::Pressed => theme.bg.emoji_btn_hover,
            Interaction::None => theme.bg.emoji_btn_idle,
        };
    }
}
```

- [ ] **Step 4.6: Update `update_checkboxes` if it writes a checked-state color directly.**

Inspect the current `update_checkboxes` body. If it writes anything to `BackgroundColor` based on `Checked` state (e.g. fills the checkbox when checked), keep that behavior but read the fill color from the theme. The natural choice: when `Checked(true)`, use `theme.bg.box_fill`; when `Checked(false)`, use `theme.bg.input` (the idle box surface). If today the system uses a different palette, preserve the current visual intent and just route the source through `theme`.

- [ ] **Step 4.7: Verify visually.**

Run: `cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
Expected: Widgets tab functions exactly as before. Click the button — count goes up, hover highlights, press depresses. Toggle the checkbox. Drag the slider. Type in the text input — focus border brightens. Open the emoji menu, click a glyph, see it append.

- [ ] **Step 4.8: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/widgets_section.rs
git commit -m "refactor(bevy-ui-showcase): migrate widgets_section.rs to theme tokens"
```

---

## Task 5: Add theme toggle button to the tab bar

**Files:**
- Modify: `cross-platform/bevy-ui-showcase/src/nav.rs`

Add a button on the right edge of the tab bar that shows a sun glyph in dark mode (means "switch to light") and a moon glyph in light mode. Clicking it triggers `handle_theme_toggle` (already registered in Task 1) which mutates the `Theme` resource and triggers all three resolvers + clear-color sync.

- [ ] **Step 5.1: In `nav.rs`, modify `spawn_tab_bar` to add a spacer + toggle button after the section tabs.**

Append this after the section-tab loop, before the final `commands.entity(bar).add_children(...)`:

```rust
    // Spacer pushes the toggle button to the right edge.
    let spacer = commands
        .spawn(Node {
            flex_grow: 1.0,
            ..default()
        })
        .id();

    // Theme toggle. The label content (sun vs moon glyph) is updated each
    // frame by `update_theme_toggle_label` so it reflects the current mode.
    let toggle = commands
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor::default(),
            BgRole::TabInactive,
            crate::theme::ThemeToggle,
        ))
        .with_child((
            Text::new("\u{f185} Light"), // initial: dark mode → offer to switch to Light
            TextColor::default(),
            TextRole::Primary,
            ThemeToggleLabel,
        ))
        .id();

    commands.entity(bar).add_children(&[spacer, toggle]);
    bar
```

Make sure to also push the section buttons before the spacer/toggle. Adjust the existing `commands.entity(bar).add_children(&buttons);` line: replace it with a single combined add_children call after the toggle is created:

```rust
    let mut all = buttons; // section tab buttons created above
    all.push(spacer);
    all.push(toggle);
    commands.entity(bar).add_children(&all);
    bar
```

- [ ] **Step 5.2: Add the `ThemeToggleLabel` marker component near the top of `nav.rs`.**

```rust
#[derive(Component)]
pub struct ThemeToggleLabel;
```

- [ ] **Step 5.3: Add a system that keeps the toggle's text in sync with the current mode.**

```rust
pub fn update_theme_toggle_label(
    theme: Res<Theme>,
    mut q: Query<&mut Text, With<ThemeToggleLabel>>,
) {
    if !theme.is_changed() {
        return;
    }
    for mut text in &mut q {
        text.0 = match theme.mode {
            crate::theme::ThemeMode::Dark => "\u{f185} Light".to_string(),  // sun glyph + offer "Light"
            crate::theme::ThemeMode::Light => "\u{f186} Dark".to_string(), // moon glyph + offer "Dark"
        };
    }
}
```

- [ ] **Step 5.4: Register `update_theme_toggle_label` in `main.rs`.**

In the second `add_systems(Update, ...)` block (the one with theme systems), add `nav::update_theme_toggle_label` to the tuple.

- [ ] **Step 5.5: Verify visually.**

Run: `cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
Expected:
1. App launches in dark mode (default). Toggle on the right of the tab bar shows " Light" (sun glyph + word).
2. Click toggle → entire app reskins to light mode in 1 frame. Toggle now shows " Dark" (moon glyph + word).
3. Switch tabs (Layout / Widgets / Theming-placeholder / Animations-placeholder) — each is in light mode.
4. Click toggle again → back to dark.
5. No flicker on tab switch, no leftover dark-mode constants visible anywhere in nav / Layout / Widgets.

- [ ] **Step 5.6: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/nav.rs cross-platform/bevy-ui-showcase/src/main.rs
git commit -m "feat(bevy-ui-showcase): theme toggle button in tab bar"
```

---

## Task 6: Bundle Inter font + scaffold `theming_section.rs`

**Files:**
- Create: `cross-platform/bevy-ui-showcase/assets/fonts/Inter-Regular.ttf`
- Create: `cross-platform/bevy-ui-showcase/assets/fonts/Inter-Bold.ttf`
- Create: `cross-platform/bevy-ui-showcase/src/theming_section.rs`
- Modify: `cross-platform/bevy-ui-showcase/src/main.rs` (replace placeholder spawn for `Section::Theming`)

- [ ] **Step 6.1: Download Inter from rsms.me/inter (OFL 1.1 licensed).**

```powershell
# From repo root.
$dest = "cross-platform/bevy-ui-showcase/assets/fonts"
# Inter v4.0 download URL — adjust if a newer release is preferred.
Invoke-WebRequest -Uri "https://github.com/rsms/inter/releases/download/v4.0/Inter-4.0.zip" -OutFile "$env:TEMP/Inter-4.0.zip"
Expand-Archive -Path "$env:TEMP/Inter-4.0.zip" -DestinationPath "$env:TEMP/Inter-4.0" -Force
Copy-Item "$env:TEMP/Inter-4.0/extras/ttf/Inter-Regular.ttf" $dest
Copy-Item "$env:TEMP/Inter-4.0/extras/ttf/Inter-Bold.ttf" $dest
```

If `Invoke-WebRequest` fails (firewall / no network), the user can manually download from `https://rsms.me/inter/` and drop `Inter-Regular.ttf` + `Inter-Bold.ttf` into `cross-platform/bevy-ui-showcase/assets/fonts/`. Either way, both files must exist before continuing.

- [ ] **Step 6.2: Create `src/theming_section.rs` scaffold.**

```rust
use bevy::prelude::*;

use crate::nav::{Section, SectionRoot};
use crate::theme::{BgRole, TextRole};

pub fn spawn(commands: &mut Commands, _asset_server: &AssetServer) -> Entity {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                display: Display::None,
                ..default()
            },
            SectionRoot(Section::Theming),
        ))
        .with_children(|root| {
            // ── Header ──
            root.spawn(Node {
                padding: UiRect::all(Val::Px(24.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|header| {
                header.spawn((
                    Text::new("Theming & Styling"),
                    TextFont { font_size: 32.0, ..default() },
                    TextColor::default(),
                    TextRole::Primary,
                ));
                header.spawn((
                    Text::new(
                        "Centralized theme tokens, role-component resolvers, \
                         and a custom font. Toggle the theme in the tab bar.",
                    ),
                    TextColor::default(),
                    TextRole::Subtle,
                ));
            });

            // ── Content column (filled by later tasks) ──
            root.spawn((
                Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(24.0)),
                    row_gap: Val::Px(24.0),
                    ..default()
                },
                ContentColumn,
            ));
        })
        .id()
}

#[derive(Component)]
pub(crate) struct ContentColumn;
```

- [ ] **Step 6.3: Wire `theming_section::spawn` into `main.rs` in place of the placeholder.**

In `main.rs`, near the top:

```rust
mod theming_section;
```

`setup_root` needs `asset_server: Res<AssetServer>` so it can pre-load the Inter handle and pass it through:

```rust
fn setup_root(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let root = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .id();

    let tab_bar = nav::spawn_tab_bar(&mut commands);
    let content = commands
        .spawn(Node {
            flex_grow: 1.0,
            width: Val::Percent(100.0),
            ..default()
        })
        .id();
    commands.entity(root).add_children(&[tab_bar, content]);

    let inter_bold: Handle<Font> = asset_server.load("fonts/Inter-Bold.ttf");

    let mut sections = Vec::with_capacity(Section::ALL.len());
    sections.push(layout_section::spawn(&mut commands));
    sections.push(widgets_section::spawn(&mut commands));
    sections.push(theming_section::spawn(&mut commands, inter_bold));
    sections.push(spawn_placeholder(&mut commands, Section::Animations));
    commands.entity(content).add_children(&sections);
}
```

The `spawn_placeholder` loop currently iterates `[Section::Theming, Section::Animations]` — replace with the single `Section::Animations` push as shown.

Update `theming_section::spawn`'s signature to take the handle directly (avoids `Res<AssetServer>` deref complications):

```rust
pub fn spawn(commands: &mut Commands, inter_bold: Handle<Font>) -> Entity {
```

Stash `inter_bold` somewhere accessible until Task 10 wires it into the typography block. Simplest: pass it through to the helper functions, or store it on a `TypographyFonts` component attached to the section root.

- [ ] **Step 6.4: Verify visually.**

Run: `cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
Expected: clicking the "Theming" tab shows a header "Theming & Styling" + subtitle. Empty content column below. Toggle theme — header recolors. Animations tab still shows the placeholder.

- [ ] **Step 6.5: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/assets/fonts/Inter-Regular.ttf cross-platform/bevy-ui-showcase/assets/fonts/Inter-Bold.ttf cross-platform/bevy-ui-showcase/src/theming_section.rs cross-platform/bevy-ui-showcase/src/main.rs
git commit -m "feat(bevy-ui-showcase): scaffold theming section + bundle Inter font"
```

---

## Task 7: Theming tab — palette swatches

**Files:**
- Modify: `cross-platform/bevy-ui-showcase/src/theming_section.rs`

A grid of swatches, one per token slot. Each swatch is a small colored square next to its slot name. Swatches recolor live when the theme toggles (because each carries the appropriate `BgRole` / `BorderRole`).

The trick: swatches must show the *resolved* color for whichever slot they represent. We can't reuse `BgRole` directly for *every* swatch (the role enum only has 9 variants, and `BgTokens` has 14 fields including hover/pressed/active/etc.). Solution: a separate marker component `Swatch` carrying a copy of the slot's name + a closure-style "which token field" identifier. Simplest approach: a `SwatchSlot` enum that mirrors every `BgTokens` / `TextTokens` / `BorderTokens` field, plus a system that runs on `theme.is_changed()` and writes the resolved color into the swatch's `BackgroundColor`.

- [ ] **Step 7.1: Define `SwatchSlot` and the swatch sync system in `theming_section.rs`.**

```rust
#[derive(Component, Clone, Copy)]
pub(crate) enum SwatchSlot {
    BgBackground, BgSurface, BgBoxFill, BgAccent,
    BgButtonIdle, BgButtonHover, BgButtonPressed,
    BgTabBar, BgTabInactive, BgTabHovered, BgTabActive,
    BgInput, BgEmojiIdle, BgEmojiHover,
    TextPrimary, TextSubtle, TextOnAccent,
    BorderSubtle, BorderFocus,
}

impl SwatchSlot {
    pub fn label(self) -> &'static str {
        match self {
            SwatchSlot::BgBackground => "bg.background",
            SwatchSlot::BgSurface => "bg.surface",
            SwatchSlot::BgBoxFill => "bg.box_fill",
            SwatchSlot::BgAccent => "bg.accent",
            SwatchSlot::BgButtonIdle => "bg.button_idle",
            SwatchSlot::BgButtonHover => "bg.button_hover",
            SwatchSlot::BgButtonPressed => "bg.button_pressed",
            SwatchSlot::BgTabBar => "bg.tab_bar",
            SwatchSlot::BgTabInactive => "bg.tab_inactive",
            SwatchSlot::BgTabHovered => "bg.tab_hovered",
            SwatchSlot::BgTabActive => "bg.tab_active",
            SwatchSlot::BgInput => "bg.input",
            SwatchSlot::BgEmojiIdle => "bg.emoji_btn_idle",
            SwatchSlot::BgEmojiHover => "bg.emoji_btn_hover",
            SwatchSlot::TextPrimary => "text.primary",
            SwatchSlot::TextSubtle => "text.subtle",
            SwatchSlot::TextOnAccent => "text.on_accent",
            SwatchSlot::BorderSubtle => "border.subtle",
            SwatchSlot::BorderFocus => "border.focus",
        }
    }

    pub fn resolve(self, theme: &crate::theme::Theme) -> Color {
        use crate::theme::Theme;
        let _ = theme; // suppress unused warning if all arms below pattern-match `theme` directly
        match self {
            SwatchSlot::BgBackground => theme.bg.background,
            SwatchSlot::BgSurface => theme.bg.surface,
            SwatchSlot::BgBoxFill => theme.bg.box_fill,
            SwatchSlot::BgAccent => theme.bg.accent,
            SwatchSlot::BgButtonIdle => theme.bg.button_idle,
            SwatchSlot::BgButtonHover => theme.bg.button_hover,
            SwatchSlot::BgButtonPressed => theme.bg.button_pressed,
            SwatchSlot::BgTabBar => theme.bg.tab_bar,
            SwatchSlot::BgTabInactive => theme.bg.tab_inactive,
            SwatchSlot::BgTabHovered => theme.bg.tab_hovered,
            SwatchSlot::BgTabActive => theme.bg.tab_active,
            SwatchSlot::BgInput => theme.bg.input,
            SwatchSlot::BgEmojiIdle => theme.bg.emoji_btn_idle,
            SwatchSlot::BgEmojiHover => theme.bg.emoji_btn_hover,
            SwatchSlot::TextPrimary => theme.text.primary,
            SwatchSlot::TextSubtle => theme.text.subtle,
            SwatchSlot::TextOnAccent => theme.text.on_accent,
            SwatchSlot::BorderSubtle => theme.border.subtle,
            SwatchSlot::BorderFocus => theme.border.focus,
        }
    }
}

pub fn update_swatches(
    theme: Res<crate::theme::Theme>,
    mut q: Query<(&SwatchSlot, &mut BackgroundColor)>,
    added: Query<(), Added<SwatchSlot>>,
) {
    if !theme.is_changed() && added.is_empty() {
        return;
    }
    for (slot, mut bg) in &mut q {
        bg.0 = slot.resolve(&theme);
    }
}
```

- [ ] **Step 7.2: Register `update_swatches` in `main.rs`.**

Add `theming_section::update_swatches` to the second `Update` system tuple.

- [ ] **Step 7.3: Spawn the palette block under the content column.**

Add a helper inside `theming_section.rs`:

```rust
fn spawn_palette_block(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|cell| {
            cell.spawn((
                Text::new("Palette"),
                TextFont { font_size: 18.0, ..default() },
                TextColor::default(),
                TextRole::Primary,
            ));

            cell.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(12.0),
                row_gap: Val::Px(12.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            })
            .with_children(|grid| {
                let slots = [
                    SwatchSlot::BgBackground, SwatchSlot::BgSurface, SwatchSlot::BgBoxFill,
                    SwatchSlot::BgAccent, SwatchSlot::BgButtonIdle, SwatchSlot::BgButtonHover,
                    SwatchSlot::BgButtonPressed, SwatchSlot::BgTabBar, SwatchSlot::BgTabInactive,
                    SwatchSlot::BgTabHovered, SwatchSlot::BgTabActive, SwatchSlot::BgInput,
                    SwatchSlot::BgEmojiIdle, SwatchSlot::BgEmojiHover, SwatchSlot::TextPrimary,
                    SwatchSlot::TextSubtle, SwatchSlot::TextOnAccent, SwatchSlot::BorderSubtle,
                    SwatchSlot::BorderFocus,
                ];
                for slot in slots {
                    grid.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(4.0),
                        width: Val::Px(96.0),
                        ..default()
                    })
                    .with_children(|swatch| {
                        swatch.spawn((
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(48.0),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor::default(),
                            BorderColor::default(),
                            crate::theme::BorderRole::Subtle,
                            slot,
                        ));
                        swatch.spawn((
                            Text::new(slot.label()),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor::default(),
                            TextRole::Subtle,
                        ));
                    });
                }
            });
        });
}
```

Then call `spawn_palette_block(&mut content)` inside the content column closure. To do that, refactor the `spawn` function to capture the content column entity and add children to it:

```rust
pub fn spawn(commands: &mut Commands, _asset_server: &AssetServer) -> Entity {
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                display: Display::None,
                ..default()
            },
            SectionRoot(Section::Theming),
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
                Text::new("Theming & Styling"),
                TextFont { font_size: 32.0, ..default() },
                TextColor::default(),
                TextRole::Primary,
            ));
            h.spawn((
                Text::new(
                    "Centralized theme tokens, role-component resolvers, \
                     and a custom font. Toggle the theme in the tab bar.",
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
            spawn_palette_block(c);
        })
        .id();

    commands.entity(root).add_children(&[header, content]);
    root
}
```

(`ContentColumn` marker is no longer needed — delete it.)

- [ ] **Step 7.4: Verify visually.**

Run: `cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
Expected: Theming tab now shows a "Palette" heading and a wrapped grid of 19 swatches, each labeled with its token name. Toggle theme — every swatch recolors live. Borders around swatches are subtle gray, also recoloring on toggle.

- [ ] **Step 7.5: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/theming_section.rs cross-platform/bevy-ui-showcase/src/main.rs
git commit -m "feat(bevy-ui-showcase): theming tab — palette swatches"
```

---

## Task 8: Theming tab — border radius row

**Files:**
- Modify: `cross-platform/bevy-ui-showcase/src/theming_section.rs`

Four panels demonstrating `RadiusTokens::sm` / `md` / `lg` / `pill`. Each panel gets `BgRole::BoxFill` so it's clearly visible.

- [ ] **Step 8.1: Add a `spawn_radius_block` helper and call it from the content column.**

```rust
fn spawn_radius_block(parent: &mut ChildSpawnerCommands, theme: &crate::theme::Theme) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|cell| {
            cell.spawn((
                Text::new("Border radius"),
                TextFont { font_size: 18.0, ..default() },
                TextColor::default(),
                TextRole::Primary,
            ));
            cell.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            })
            .with_children(|row| {
                let entries = [
                    ("sm", theme.radius.sm),
                    ("md", theme.radius.md),
                    ("lg", theme.radius.lg),
                    ("pill", theme.radius.pill),
                ];
                for (name, radius) in entries {
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|cell| {
                        cell.spawn((
                            Node {
                                width: Val::Px(72.0),
                                height: Val::Px(72.0),
                                border_radius: BorderRadius::all(Val::Px(radius)),
                                ..default()
                            },
                            BackgroundColor::default(),
                            BgRole::BoxFill,
                        ));
                        cell.spawn((
                            Text::new(name),
                            TextFont { font_size: 12.0, ..default() },
                            TextColor::default(),
                            TextRole::Subtle,
                        ));
                    });
                }
            });
        });
}
```

- [ ] **Step 8.2: Wire it into the content column.**

The closure passed to `with_children` does not have access to `theme` directly. Two options:
1. Make `spawn` take `theme: &Theme` as a parameter and forward it. Cleanest.
2. Read the radius values from a default `Theme` (they're constant across modes anyway — radius doesn't change between dark/light). Simplest.

Use option 2 — just call `crate::theme::Theme::default()` inline inside `spawn_radius_block`:

```rust
fn spawn_radius_block(parent: &mut ChildSpawnerCommands) {
    let theme = crate::theme::Theme::default();
    // … rest as above, using `theme.radius.sm` etc.
}
```

Then call `spawn_radius_block(c);` inside the content `with_children` closure, after `spawn_palette_block(c);`.

- [ ] **Step 8.3: Verify visually.**

Run: `cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
Expected: Theming tab now shows palette grid + a "Border radius" heading + a row of four colored squares. The first three have progressively rounder corners; the fourth is a circle (`pill`). Each labeled `sm` / `md` / `lg` / `pill`. Toggle theme — squares recolor, but radii stay the same.

- [ ] **Step 8.4: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/theming_section.rs
git commit -m "feat(bevy-ui-showcase): theming tab — border radius row"
```

---

## Task 9: Theming tab — border color & width row

**Files:**
- Modify: `cross-platform/bevy-ui-showcase/src/theming_section.rs`

Four bordered panels: subtle thin (1px), subtle thick (3px), focus thin, focus thick. Each panel uses `BgRole::Surface` for its inside and the appropriate `BorderRole` for its border.

- [ ] **Step 9.1: Add `spawn_border_block` and call from content column.**

```rust
fn spawn_border_block(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|cell| {
            cell.spawn((
                Text::new("Border color & width"),
                TextFont { font_size: 18.0, ..default() },
                TextColor::default(),
                TextRole::Primary,
            ));
            cell.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            })
            .with_children(|row| {
                let entries: [(&str, crate::theme::BorderRole, f32); 4] = [
                    ("subtle 1px", crate::theme::BorderRole::Subtle, 1.0),
                    ("subtle 3px", crate::theme::BorderRole::Subtle, 3.0),
                    ("focus 1px", crate::theme::BorderRole::Focus, 1.0),
                    ("focus 3px", crate::theme::BorderRole::Focus, 3.0),
                ];
                for (name, role, width) in entries {
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|cell| {
                        cell.spawn((
                            Node {
                                width: Val::Px(96.0),
                                height: Val::Px(64.0),
                                border: UiRect::all(Val::Px(width)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor::default(),
                            BgRole::Surface,
                            BorderColor::default(),
                            role,
                        ));
                        cell.spawn((
                            Text::new(name),
                            TextFont { font_size: 12.0, ..default() },
                            TextColor::default(),
                            TextRole::Subtle,
                        ));
                    });
                }
            });
        });
}
```

Add `spawn_border_block(c);` in the content `with_children` closure after `spawn_radius_block(c);`.

- [ ] **Step 9.2: Verify visually.**

Run: `cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
Expected: Theming tab now shows palette + radius + "Border color & width" heading + a row of four bordered panels, two with subtle borders (one thin, one thick) and two with focus borders. Inside fill is `surface` (slightly different from page background). Toggle theme — borders and fills recolor.

- [ ] **Step 9.3: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/theming_section.rs
git commit -m "feat(bevy-ui-showcase): theming tab — border color & width row"
```

---

## Task 10: Theming tab — typography sample with Inter

**Files:**
- Modify: `cross-platform/bevy-ui-showcase/src/theming_section.rs`

Three text rows: a heading using Inter Bold 28px, body using the global default (HackNerdFont) at 14px, mono using HackNerdFont at 12px. All three carry `TextRole::Primary` so they recolor with theme.

- [ ] **Step 10.1: Add `spawn_typography_block` that loads Inter via `AssetServer`.**

```rust
fn spawn_typography_block(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    let inter_bold: Handle<Font> = asset_server.load("fonts/Inter-Bold.ttf");

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|cell| {
            cell.spawn((
                Text::new("Typography"),
                TextFont { font_size: 18.0, ..default() },
                TextColor::default(),
                TextRole::Primary,
            ));
            cell.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            })
            .with_children(|stack| {
                stack.spawn((
                    Text::new("Heading — Inter Bold, 28px"),
                    TextFont { font: inter_bold.clone(), font_size: 28.0, ..default() },
                    TextColor::default(),
                    TextRole::Primary,
                ));
                stack.spawn((
                    Text::new("Body — HackNerdFont (default), 14px"),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor::default(),
                    TextRole::Primary,
                ));
                stack.spawn((
                    Text::new("Mono — HackNerdFont, 12px (mono by nature)"),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor::default(),
                    TextRole::Subtle,
                ));
            });
        });
}
```

- [ ] **Step 10.2: Wire `asset_server` through `spawn` to `spawn_typography_block`.**

The `spawn` function already takes `asset_server: &AssetServer`. Pass it through:

```rust
let content = commands
    .spawn(Node { /* … */ })
    .with_children(|c| {
        spawn_palette_block(c);
        spawn_radius_block(c);
        spawn_border_block(c);
        spawn_typography_block(c, asset_server);
    })
    .id();
```

- [ ] **Step 10.3: Verify visually.**

Run: `cargo run --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml`
Expected: Theming tab now ends with a "Typography" heading + three rows of text. The first row visibly uses a *different font* (Inter — proportional, smoother, not monospace). The second and third use HackNerdFont. Toggle theme — text colors invert correctly. The Inter font may load 1 frame after spawn (asset load is async) — first frame shows fallback, subsequent frames show Inter. That's expected.

- [ ] **Step 10.4: Commit.**

```powershell
git add cross-platform/bevy-ui-showcase/src/theming_section.rs
git commit -m "feat(bevy-ui-showcase): theming tab — typography sample with Inter"
```

---

## Task 11: Final validation

**Files:** none (verification only)

- [ ] **Step 11.1: Clippy clean.**

Run: `cargo clippy --manifest-path cross-platform/bevy-ui-showcase/Cargo.toml -- -D warnings`
Expected: no errors, no warnings.

If there are warnings about unused token fields (e.g., `bg.button_hover`), they should already be used by the migrated widget systems — investigate any unused field warnings and either remove the field if truly unused or wire it up.

- [ ] **Step 11.2: No raw `Color::srgb` outside `theme.rs`.**

Run a grep:
```powershell
# In repo root.
Select-String -Path "cross-platform/bevy-ui-showcase/src/*.rs" -Pattern "Color::srgb" | Where-Object { $_.Path -notlike "*theme.rs" }
```
Expected: no matches. The only `Color::srgb` literals in the bevy-ui-showcase crate should be in `src/theme.rs`'s `Theme::dark()` / `Theme::light()` constructors.

If any matches are found in `nav.rs`, `layout_section.rs`, `widgets_section.rs`, or `theming_section.rs`, replace them with theme-token role components (see migration rules in Tasks 2-4 and the spec's migration table).

Acceptable exception: `Color::WHITE` / `Color::BLACK` constants used in transient places where they truly don't belong to the theme (e.g., a debug placeholder). None should remain after the migration; flag any to the user.

- [ ] **Step 11.3: Visual sweep.**

Run the app and walk through:
1. App launches in dark mode; tab bar, Layout, Widgets all look identical to before Phase 4.
2. Click Theming tab — palette swatches, radius row, border row, typography sample all visible.
3. Click theme toggle in tab bar — entire app reskins to light mode (header text, surfaces, swatches, borders, body text, all four tabs).
4. Click each tab in light mode — Layout, Widgets, Theming all recolor; Animations placeholder also recolors.
5. Click toggle again — back to dark mode. Tab switching, button presses, slider drag, checkbox toggle, text input focus, emoji menu — everything works in both modes.

- [ ] **Step 11.4: Update Phase 4 task status.**

The TaskList in this session should already track Phase 4. Mark it `completed` after the visual sweep passes.

- [ ] **Step 11.5: Commit any small fixups from clippy / visual sweep.**

If steps 11.1 / 11.2 / 11.3 surfaced anything to fix, commit it as `fix(bevy-ui-showcase): post-Phase-4 cleanup`. Otherwise no commit needed.

---

## Out of scope reminder

The spec deliberately defers: animated theme transitions (Phase 5), persistence across launches, OS-theme detection, per-component overrides, WCAG contrast verification, and spawn helpers to eliminate the 1-frame flash. If any of these come up during implementation, file a follow-up rather than scope-creeping this plan.
