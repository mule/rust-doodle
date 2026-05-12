use bevy::prelude::*;

use crate::theme::{BgRole, TextRole, Theme, ThemeTransition};

#[derive(Component)]
pub struct ThemeToggleLabel;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Layout,
    Widgets,
    Theming,
    Animations,
}

impl Section {
    pub fn label(self) -> &'static str {
        match self {
            Section::Layout => "Layout",
            Section::Widgets => "Widgets",
            Section::Theming => "Theming",
            Section::Animations => "Animations",
        }
    }

    pub const ALL: [Section; 4] = [
        Section::Layout,
        Section::Widgets,
        Section::Theming,
        Section::Animations,
    ];
}

#[derive(Resource)]
pub struct CurrentSection(pub Section);

impl Default for CurrentSection {
    fn default() -> Self {
        Self(Section::Layout)
    }
}

#[derive(Component)]
pub struct SectionRoot(pub Section);

#[derive(Component)]
pub struct TabButton(pub Section);

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
                // No BgRole: update_tab_visuals owns the BackgroundColor end-to-end
                // (it needs to drive (inactive, hovered, active) from the same source
                // of truth, and runs every frame). Carrying BgRole::TabInactive here
                // would race with the resolver and effectively be dead.
                BackgroundColor::default(),
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

    let mut all = buttons; // section tab buttons created above
    all.push(spacer);
    all.push(toggle);
    commands.entity(bar).add_children(&all);
    bar
}

pub fn handle_tab_clicks(
    mut current: ResMut<CurrentSection>,
    query: Query<(&Interaction, &TabButton), Changed<Interaction>>,
) {
    for (interaction, tab) in &query {
        if *interaction == Interaction::Pressed {
            current.0 = tab.0;
        }
    }
}

pub fn update_section_visibility(
    current: Res<CurrentSection>,
    mut query: Query<(&SectionRoot, &mut Node)>,
) {
    if !current.is_changed() {
        return;
    }
    for (root, mut node) in &mut query {
        node.display = if root.0 == current.0 {
            Display::Flex
        } else {
            Display::None
        };
    }
}

/// Paint every tab button's background based on (active, interaction) state.
/// Owns the tab BackgroundColor end-to-end — tabs carry no BgRole, so this
/// is the sole writer. Runs every frame (cheap at ~4 entities); blends the
/// three tab colors through `ThemeTransition` so toggling theme crossfades
/// the tabs alongside the rest of the app.
pub fn update_tab_visuals(
    theme: Res<Theme>,
    transition: Option<Res<ThemeTransition>>,
    current: Res<CurrentSection>,
    mut query: Query<(&Interaction, &TabButton, &mut BackgroundColor)>,
) {
    use bevy::color::Mix;
    let (inactive, hovered, active) = if let Some(t) = transition.as_ref() {
        let p = t.eased_progress();
        (
            t.from_bg.tab_inactive.mix(&theme.bg.tab_inactive, p),
            t.from_bg.tab_hovered.mix(&theme.bg.tab_hovered, p),
            t.from_bg.tab_active.mix(&theme.bg.tab_active, p),
        )
    } else {
        (theme.bg.tab_inactive, theme.bg.tab_hovered, theme.bg.tab_active)
    };
    for (interaction, tab, mut bg) in &mut query {
        let is_active = tab.0 == current.0;
        bg.0 = match (is_active, *interaction) {
            (true, _) => active,
            (false, Interaction::Hovered | Interaction::Pressed) => hovered,
            (false, Interaction::None) => inactive,
        };
    }
}

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
            crate::theme::ThemeMode::Light => "\u{f186} Dark".to_string(),  // moon glyph + offer "Dark"
        };
    }
}
