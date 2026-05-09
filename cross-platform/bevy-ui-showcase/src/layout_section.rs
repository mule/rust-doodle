use bevy::prelude::*;

use crate::nav::{Section, SectionRoot};
use crate::theme::{BgRole, TextRole};

/// Tag on a percent-box's `Text`: lets `update_percent_labels` find both the
/// flex-grown box and its row each frame and recompute the displayed share.
#[derive(Component)]
pub(crate) struct PercentBoxLabel {
    row: Entity,
    box_node: Entity,
}

pub fn spawn(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            SectionRoot(Section::Layout),
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
                    Text::new("Layout & Flexbox"),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor::default(),
                    TextRole::Primary,
                ));
                header.spawn((
                    Text::new("Bevy UI's Node component drives flexbox-style layout."),
                    TextColor::default(),
                    TextRole::Subtle,
                ));
            });

            // ── Demos column ──
            root.spawn(Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(24.0),
                ..default()
            })
            .with_children(|demos| {
                //
                // ── Demo 1 (provided): FlexDirection::Row + FlexGrow ──
                //
                demos
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|cell| {
                        cell.spawn((
                            Text::new("FlexDirection::Row + FlexGrow"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor::default(),
                            TextRole::Primary,
                        ));
                        cell.spawn((
                            Text::new(
                                "Three boxes in a row; the middle one has flex_grow: 1.0 \
                                 so it absorbs remaining space.",
                            ),
                            TextColor::default(),
                            TextRole::Subtle,
                        ));
                        cell.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                padding: UiRect::all(Val::Px(8.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor::default(),
                            BgRole::Surface,
                        ))
                        .with_children(|row| {
                            row.spawn((
                                Node {
                                    width: Val::Px(100.0),
                                    height: Val::Px(60.0),
                                    ..default()
                                },
                                BackgroundColor::default(),
                                BgRole::BoxFill,
                            ));
                            row.spawn((
                                Node {
                                    flex_grow: 1.0,
                                    height: Val::Px(60.0),
                                    ..default()
                                },
                                BackgroundColor::default(),
                                BgRole::Accent,
                            ));
                            row.spawn((
                                Node {
                                    width: Val::Px(100.0),
                                    height: Val::Px(60.0),
                                    ..default()
                                },
                                BackgroundColor::default(),
                                BgRole::BoxFill,
                            ));
                        });
                    });

                //
                // ── Demo 2 (your turn) ──
                //
                // Pick one of:
                //   • JustifyContent — fixed-size boxes spaced via SpaceBetween / Center /
                //     SpaceAround on the parent's justify_content.
                //   • AlignItems on the cross axis — boxes of different heights aligned
                //     Start / Center / End via the parent's align_items.
                //   • Percentage vs pixel sizing — two children, one Val::Percent(50.0),
                //     one Val::Px(200.0). Resize: only the percent box reflows.
                //   • Nested column inside a row — main axis switches mid-tree.
                //
                // Edit the title / description below and add `row.spawn(...)` calls in the
                // marked closure.
                //
                demos
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|cell| {
                        cell.spawn((
                            Text::new("Demo 2"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor::default(),
                            TextRole::Primary,
                        ));
                        cell.spawn((
                            Text::new("Nested column inside a row"),
                            TextColor::default(),
                            TextRole::Subtle,
                        ));
                        cell.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                padding: UiRect::all(Val::Px(8.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                // TODO: tweak this Node's flexbox properties for your demo.
                                // e.g. `justify_content: JustifyContent::SpaceBetween,`
                                ..default()
                            },
                            BackgroundColor::default(),
                            BgRole::Surface,
                        ))
                        .with_children(|row| {
                            // Three "cards" sit side-by-side in this Row. Each card is
                            // itself a Column — that's the axis switch: outer flow is
                            // horizontal, but inside each card the flow is vertical.
                            for i in 0..3 {
                                row.spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(4.0),
                                    ..default()
                                })
                                .with_children(|card| {
                                    // Accent strip on top (with a label), body block
                                    // below — stacked because this card is column-flex.
                                    card.spawn((
                                        Node {
                                            width: Val::Px(120.0),
                                            height: Val::Px(20.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor::default(),
                                        BgRole::Accent,
                                    ))
                                    .with_child((
                                        Text::new(format!("Card {}", i + 1)),
                                        TextFont {
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor::default(),
                                        TextRole::OnAccent,
                                    ));
                                    card.spawn((
                                        Node {
                                            width: Val::Px(120.0),
                                            height: Val::Px(60.0),
                                            ..default()
                                        },
                                        BackgroundColor::default(),
                                        BgRole::BoxFill,
                                    ));
                                });
                            }
                        });
                    });

                //
                // ── Demo 3 ──
                //
                demos
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|cell| {
                        cell.spawn((
                            Text::new("Demo 3 - pixel vs percentage sizing"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor::default(),
                            TextRole::Primary,
                        ));
                        cell.spawn((
                            Text::new("This demo illustrates the difference between pixel and percentage-based sizing in a flex container."),
                            TextColor::default(),
                            TextRole::Subtle,
                        ));
                        cell.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                padding: UiRect::all(Val::Px(8.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor::default(),
                            BgRole::Surface,
                        ))
                        .with_children(|row| {
                            // Capture the row's entity so percent-box labels can ask the
                            // layout solver each frame "what fraction of this row am I?".
                            let row_id = row.target_entity();
                            const TOTAL_NODES: usize = 6;

                            for i in 0..TOTAL_NODES {
                                let is_pixel = i % 2 == 0;
                                // Pixel boxes hold their declared width; percent boxes use
                                // `flex_grow: 1.0` so they each absorb an equal share of
                                // whatever remains after the pixel boxes are placed.
                                let node = if is_pixel {
                                    Node {
                                        width: Val::Px(200.0),
                                        flex_shrink: 0.0,
                                        height: Val::Px(60.0),
                                        ..default()
                                    }
                                } else {
                                    Node {
                                        flex_grow: 1.0,
                                        height: Val::Px(60.0),
                                        ..default()
                                    }
                                };
                                let bg_role = if is_pixel { BgRole::BoxFill } else { BgRole::Accent };
                                let mut box_cmds = row.spawn((node, BackgroundColor::default(), bg_role));
                                let box_id = box_cmds.id();

                                box_cmds.with_children(|inner| {
                                    let initial_label = if is_pixel {
                                        "Pixel (200px)".to_string()
                                    } else {
                                        // Placeholder — overwritten on the first frame
                                        // by `update_percent_labels` once layout runs.
                                        "Percentage (—)".to_string()
                                    };
                                    let mut text_cmds = inner.spawn((
                                        Text::new(initial_label),
                                        TextFont {
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor::default(),
                                        TextRole::OnAccent,
                                    ));
                                    if !is_pixel {
                                        text_cmds.insert(PercentBoxLabel {
                                            row: row_id,
                                            box_node: box_id,
                                        });
                                    }
                                });
                            }
                        });
                    });
            });
        })
        .id()
}

/// Each frame, divide every percent-box's computed width by its row's computed
/// width and write the result into the box's label. Runs in `Update`, so the
/// `ComputedNode` values it reads are from last frame's layout pass — that's a
/// one-frame lag, invisible at 60 fps.
pub fn update_percent_labels(
    nodes: Query<&ComputedNode>,
    mut labels: Query<(&PercentBoxLabel, &mut Text)>,
) {
    for (label, mut text) in &mut labels {
        let Ok(row_node) = nodes.get(label.row) else {
            continue;
        };
        let Ok(box_node) = nodes.get(label.box_node) else {
            continue;
        };
        if row_node.size.x <= 0.0 {
            continue;
        }
        let pct = box_node.size.x / row_node.size.x * 100.0;
        text.0 = format!("Percentage ({:.1}%)", pct);
    }
}
