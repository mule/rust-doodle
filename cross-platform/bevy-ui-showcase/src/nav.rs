use bevy::prelude::*;

use crate::theme::{BgRole, TextRole, Theme};

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
