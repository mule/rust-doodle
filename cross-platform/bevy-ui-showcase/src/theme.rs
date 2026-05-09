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
        *color = BorderColor::all(theme.border.resolve(*role));
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
