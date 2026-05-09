use bevy::prelude::*;

use crate::nav::{Section, SectionRoot};
use crate::theme::TextRole;

pub fn spawn(commands: &mut Commands, _inter_bold: Handle<Font>) -> Entity {
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
        .id();

    commands.entity(root).add_children(&[header, content]);
    root
}
