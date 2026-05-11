use bevy::math::curve::EaseFunction;
use bevy::prelude::*;

use crate::nav::{Section, SectionRoot};
use crate::theme::{BgRole, BorderRole, TextRole};
use crate::theme::Theme;
use crate::tween::Tween;
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
        })
        .id();

    commands.entity(root).add_children(&[header, content]);
    root
}

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
