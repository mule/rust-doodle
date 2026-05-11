use bevy::prelude::*;

use crate::nav::{Section, SectionRoot};
use crate::theme::{BgRole, TextRole};
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
