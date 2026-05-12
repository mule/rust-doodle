use bevy::prelude::*;

use crate::nav::{Section, SectionRoot};
use crate::theme::{BorderRole, TextRole};

pub fn spawn(commands: &mut Commands, inter_bold: Handle<Font>) -> Entity {
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
            // ── Palette block ──
            c.spawn(Node {
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
                        SwatchSlot::BgBackground,
                        SwatchSlot::BgSurface,
                        SwatchSlot::BgBoxFill,
                        SwatchSlot::BgAccent,
                        SwatchSlot::BgButtonIdle,
                        SwatchSlot::BgButtonHover,
                        SwatchSlot::BgButtonPressed,
                        SwatchSlot::BgTabBar,
                        SwatchSlot::BgTabInactive,
                        SwatchSlot::BgTabHovered,
                        SwatchSlot::BgTabActive,
                        SwatchSlot::BgInput,
                        SwatchSlot::BgEmojiIdle,
                        SwatchSlot::BgEmojiHover,
                        SwatchSlot::BgSliderTrack,
                        SwatchSlot::BgSliderThumb,
                        SwatchSlot::TextPrimary,
                        SwatchSlot::TextSubtle,
                        SwatchSlot::TextOnAccent,
                        SwatchSlot::BorderSubtle,
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
                                BorderRole::Subtle,
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

            // Border radius row: four panels using sm/md/lg/pill from RadiusTokens.
            // Radii are constant across themes (light/dark don't change radii), so
            // we read from a default Theme instance here. This avoids needing to
            // pipe Res<Theme> through the spawn function.
            let radii = crate::theme::Theme::default().radius;
            c.spawn(Node {
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
                        ("sm", radii.sm),
                        ("md", radii.md),
                        ("lg", radii.lg),
                        ("pill", radii.pill),
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
                                crate::theme::BgRole::BoxFill,
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

            // Border color & width row: four bordered panels demonstrating
            // BorderRole::Subtle vs BorderRole::Focus at 1px and 3px widths.
            // Inside fill is BgRole::Surface so the border has visible contrast.
            c.spawn(Node {
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
                                crate::theme::BgRole::Surface,
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

            // Typography sample. Inter (proportional, OFL-licensed) for the
            // heading shows visual contrast against HackNerdFont (monospace)
            // which is the global default for body and mono rows.
            // The Inter handle is captured into this closure from spawn's parameter.
            let inter_bold_clone = inter_bold.clone();
            c.spawn(Node {
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
                        TextFont { font: inter_bold_clone.clone(), font_size: 28.0, ..default() },
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
        })
        .id();

    commands.entity(root).add_children(&[header, content]);
    root
}

// ── SwatchSlot ───────────────────────────────────────────────────────────────

#[derive(Component, Clone, Copy)]
pub(crate) enum SwatchSlot {
    BgBackground,
    BgSurface,
    BgBoxFill,
    BgAccent,
    BgButtonIdle,
    BgButtonHover,
    BgButtonPressed,
    BgTabBar,
    BgTabInactive,
    BgTabHovered,
    BgTabActive,
    BgInput,
    BgEmojiIdle,
    BgEmojiHover,
    BgSliderTrack,
    BgSliderThumb,
    TextPrimary,
    TextSubtle,
    TextOnAccent,
    BorderSubtle,
    BorderFocus,
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
            SwatchSlot::BgSliderTrack => "bg.slider_track",
            SwatchSlot::BgSliderThumb => "bg.slider_thumb",
            SwatchSlot::TextPrimary => "text.primary",
            SwatchSlot::TextSubtle => "text.subtle",
            SwatchSlot::TextOnAccent => "text.on_accent",
            SwatchSlot::BorderSubtle => "border.subtle",
            SwatchSlot::BorderFocus => "border.focus",
        }
    }

    pub fn resolve(self, theme: &crate::theme::Theme) -> Color {
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
            SwatchSlot::BgSliderTrack => theme.bg.slider_track,
            SwatchSlot::BgSliderThumb => theme.bg.slider_thumb,
            SwatchSlot::TextPrimary => theme.text.primary,
            SwatchSlot::TextSubtle => theme.text.subtle,
            SwatchSlot::TextOnAccent => theme.text.on_accent,
            SwatchSlot::BorderSubtle => theme.border.subtle,
            SwatchSlot::BorderFocus => theme.border.focus,
        }
    }
}

// ── update_swatches system ───────────────────────────────────────────────────

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
