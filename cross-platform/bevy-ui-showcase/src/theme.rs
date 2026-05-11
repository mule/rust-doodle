use bevy::math::curve::EaseFunction;
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
    pub slider_track: Color,
    pub slider_thumb: Color,
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
                slider_track: Color::srgb(0.20, 0.22, 0.28),
                slider_thumb: Color::srgb(0.486, 0.188, 0.188),
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
                slider_track: Color::srgb(0.80, 0.81, 0.84),
                slider_thumb: Color::srgb(0.65, 0.30, 0.30),
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

// ── Role enum components ────────────────────────────────────────────────────

#[derive(Component, Clone, Copy, Debug)]
pub enum BgRole {
    Surface,
    BoxFill,
    Accent,
    ButtonIdle,
    TabBar,
    TabInactive,
    Input,
    EmojiBtnIdle,
    SliderTrack,
    SliderThumb,
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
            BgRole::Surface => self.surface,
            BgRole::BoxFill => self.box_fill,
            BgRole::Accent => self.accent,
            BgRole::ButtonIdle => self.button_idle,
            BgRole::TabBar => self.tab_bar,
            BgRole::TabInactive => self.tab_inactive,
            BgRole::Input => self.input,
            BgRole::EmojiBtnIdle => self.emoji_btn_idle,
            BgRole::SliderTrack => self.slider_track,
            BgRole::SliderThumb => self.slider_thumb,
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

// ── Clear-color sync ────────────────────────────────────────────────────────

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

// ── Transition advance system ────────────────────────────────────────────────

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
