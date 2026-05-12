use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::math::curve::EaseFunction;
use bevy::prelude::*;
use bevy::window::{CursorIcon, SystemCursorIcon};

use crate::nav::{Section, SectionRoot};
use crate::theme::{BgRole, BorderRole, TextRole, Theme, ThemeTransition};
use crate::tween::Tween;

/// Per-button click state. Lives on the Button entity itself.
#[derive(Component, Default)]
pub(crate) struct ClickCount(pub u32);

/// Tag on the Text *inside* a counter button so the click handler can find it
/// among the button's children and rewrite it after each click.
#[derive(Component)]
pub(crate) struct ClickCountLabel;

/// Checkbox state — lives on the clickable box entity.
#[derive(Component, Default)]
pub(crate) struct Checked(pub bool);

#[derive(Component, Default)]
pub(crate) struct SliderValue(pub f32);

#[derive(Component)]
pub(crate) struct Slider;

#[derive(Component)]
pub(crate) struct SliderThumb;

/// Tag on a Text that should show a slider's live value. Holds the track entity
/// so the update system knows which `SliderValue` to read.
#[derive(Component)]
pub(crate) struct SliderValueLabel {
    pub slider: Entity,
}

/// Marker on a clickable text-input field.
#[derive(Component)]
pub(crate) struct TextInput;

/// The buffered text contents of a `TextInput` field.
#[derive(Component, Default)]
pub(crate) struct TextInputBuffer(pub String);

/// Tag on the child `Text` inside a text-input field that displays the buffer
/// (or a placeholder when empty + unfocused).
#[derive(Component)]
pub(crate) struct TextInputDisplay;

/// Currently-focused text input, if any. Set on Pressed; cleared on Escape or
/// on click outside (via Pressed of any non-input UI element — out of scope
/// here, so we only handle Escape).
#[derive(Resource, Default)]
pub(crate) struct FocusedTextInput(pub Option<Entity>);

/// Maximum buffer length — protects against runaway typing.
const TEXT_INPUT_MAX_LEN: usize = 64;

/// Toggle button that shows/hides the popup icon menu.
#[derive(Component)]
pub(crate) struct EmojiMenuButton;

/// Container for the popup icon menu. We toggle its `Node.display` between
/// `None` and `Flex` to show/hide.
#[derive(Component)]
pub(crate) struct EmojiMenu;

/// Each clickable item in the menu carries the character it should insert
/// into the currently-focused text input.
#[derive(Component)]
pub(crate) struct EmojiMenuItem(pub char);

/// HackNerdFont code points for a 4×3 grid of "fun" icons. All in the Nerd
/// Font Private Use Area, so they render in any HackNerdFont-using Text.
const EMOJI_ICONS: [char; 12] = [
    '\u{f004}', // fa-heart
    '\u{f005}', // fa-star
    '\u{f164}', // fa-thumbs-up
    '\u{f165}', // fa-thumbs-down
    '\u{f118}', // fa-smile-o
    '\u{f119}', // fa-frown-o
    '\u{f11a}', // fa-meh-o
    '\u{f0e7}', // fa-bolt
    '\u{f135}', // fa-rocket
    '\u{f0f4}', // fa-coffee
    '\u{f1b0}', // fa-paw
    '\u{f06d}', // fa-fire
];

/// Tag on the child Text inside a checkbox: the system rewrites it to "✓" or "".
#[derive(Component)]
pub(crate) struct CheckboxGlyph;

pub fn spawn(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                display: Display::None, // hidden until the Widgets tab is selected
                ..default()
            },
            SectionRoot(Section::Widgets),
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
                    Text::new("Widgets & Interactions"),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor::default(),
                    TextRole::Primary,
                ));
                header.spawn((
                    Text::new(
                        "Bevy ships only the Button + Interaction primitives — \
                         every other widget is something you build.",
                    ),
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
                // ── Demo 1 (provided): click-counter button ──
                //
                // Spawns a clickable button with a count label inside it. Each click
                // bumps the ClickCount and `update_click_buttons` rewrites the label.
                //
                demos
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|cell| {
                        cell.spawn((
                            Text::new("Button + persistent state"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor::default(),
                            TextRole::Primary,
                        ));
                        cell.spawn((
                            Text::new(
                                "Click the button. Its label is rewritten by a system \
                                 that owns no state — the count lives on the button entity.",
                            ),
                            TextColor::default(),
                            TextRole::Subtle,
                        ));
                        cell.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(12.0),
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
                                Text::new("Clicked: 0"),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor::default(),
                                TextRole::Primary,
                                ClickCountLabel,
                            ));
                        });
                    });

                //
                // ── Demo 2 (your turn): checkbox ──
                //
                // The widget shape:
                //   • A small square (~20px) — the visual "box".
                //   • A child Text inside it that's "✓" when checked, "" when not.
                //   • A `Checked(bool)` state component on the box entity.
                //   • A click system that flips the bool on `Interaction::Pressed`.
                //   • A render system that updates the child Text + maybe the box color.
                //
                // Decisions you'll make:
                //   • Click target — Button on the square itself, or a wrapping label?
                //   • Visual — checkmark glyph vs. fill-when-checked vs. both?
                //   • Default state — checked or unchecked?
                //
                // Add the Checked component, the spawn calls, and (in main.rs) wire up
                // the click-handler system you'll write below.
                //
                demos
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|cell| {
                        cell.spawn((
                            Text::new("Checkbox + boolean state"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor::default(),
                            TextRole::Primary,
                        ));
                        cell.spawn((
                            Text::new(
                                "Click the square to toggle. The glyph child flips between \
                                 \"\" and \"\u{f00c}\" (Nerd Font fa-check) - same \
                                 children-walk pattern as the click counter, applied to \
                                 a boolean instead of a number.",
                            ),
                            TextColor::default(),
                            TextRole::Subtle,
                        ));
                        cell.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(12.0),
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(12.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor::default(),
                            BgRole::Surface,
                        ))
                        .with_children(|row| {
                            // The clickable square. `Button` makes it pick up `Interaction`
                            // events; `Checked` carries the bool state.
                            row.spawn((
                                Button,
                                Node {
                                    width: Val::Px(20.0),
                                    height: Val::Px(20.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(2.0)),
                                    border_radius: BorderRadius::all(Val::Px(3.0)),
                                    ..default()
                                },
                                BackgroundColor::default(),
                                BgRole::Input,
                                BorderColor::default(),
                                BorderRole::Subtle,
                                Checked::default(),
                            ))
                            .with_child((
                                // Starts empty; the toggle system rewrites this each click.
                                Text::new(""),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor::default(),
                                TextRole::Primary,
                                CheckboxGlyph,
                            ));

                            // Plain label next to the checkbox. Not clickable (per the
                            // square-only click target choice).
                            row.spawn((
                                Text::new("Toggle me"),
                                TextColor::default(),
                                TextRole::Primary,
                            ));
                        });
                    });

                //
                // ── Demo 3 (your turn): slider ──
                //
                // Hardest of the four. The shape:
                //   • A "track" Node (e.g. 200×8px, rounded).
                //   • A "thumb" Node child positioned absolutely along the track.
                //   • A `SliderValue(f32)` (0.0..=1.0) on the track entity.
                //   • A `Slider` tag so a system can find them all.
                //   • A drag system: on `Interaction::Pressed`, read the cursor's x
                //     relative to the track's ComputedNode rect and write to value.
                //
                // Decisions you'll make:
                //   • Range (0..1, or labelled units like 0..100)?
                //   • Click-to-set or only drag, or both?
                //   • Show numeric value next to the slider?
                //
                demos
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|cell| {
                        cell.spawn((
                            Text::new("Slider + drag interaction"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor::default(),
                            TextRole::Primary,
                        ));
                        cell.spawn((
                            Text::new("Drag the thumb. The track's SliderValue is updated by a system that reads cursor position each frame while the track is pressed."),
                            TextColor::default(),
                            TextRole::Subtle,
                        ));
                        cell.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(12.0),
                                align_items: AlignItems::Center, // center min/track/max/value vertically
                                padding: UiRect::all(Val::Px(12.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor::default(),
                            BgRole::Surface,
                        ))
                        .with_children(|row| {
                            // Min-value label on the left.
                            row.spawn((
                                Text::new("0.0"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor::default(),
                                TextRole::Subtle,
                            ));

                            // Track + thumb. Capture the track's entity ID so the
                            // value-display label below can point at it.
                            let track_id = row
                                .spawn((
                                    // Button triggers required-component injection of
                                    // Interaction + FocusPolicy so the track is pickable.
                                    Button,
                                    Node {
                                        width: Val::Px(200.0),
                                        height: Val::Px(8.0),
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor::default(),
                                    BgRole::SliderTrack,
                                    SliderValue(0.5),
                                    Slider,
                                ))
                                .with_child((
                                    Node {
                                        width: Val::Px(16.0),
                                        height: Val::Px(16.0),
                                        position_type: PositionType::Absolute,
                                        left: Val::Percent(50.0),
                                        top: Val::Percent(-50.0),
                                        border_radius: BorderRadius::all(Val::Px(24.0)),
                                        ..default()
                                    },
                                    BackgroundColor::default(),
                                    BgRole::SliderThumb,
                                    SliderThumb,
                                ))
                                .id();

                            // Max-value label on the right.
                            row.spawn((
                                Text::new("1.0"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor::default(),
                                TextRole::Subtle,
                            ));

                            // Live value display. `SliderValueLabel` ties this Text
                            // to the track so `update_slider_value_labels` can rewrite
                            // it whenever that track's `SliderValue` changes.
                            row.spawn((
                                Text::new("value: 0.50"),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor::default(),
                                TextRole::Primary,
                                SliderValueLabel { slider: track_id },
                            ));
                        });
                    });

                //
                // ── Demo 4 (your turn): text input ──
                //
                // Bevy has no built-in text editor, so this is the most "from scratch"
                // widget. The shape:
                //   • A bordered Node sized like an input field.
                //   • A child Text that holds the current buffer.
                //   • A `TextInputBuffer(String)` + `TextInputFocused(bool)` on the field.
                //   • A click system that toggles focus on Pressed.
                //   • A keyboard system that reads `Res<ButtonInput<KeyCode>>` (for
                //     things like Backspace) and `EventReader<KeyboardInput>` (for the
                //     printable characters via `KeyboardInput::logical_key`).
                //
                // Decisions you'll make:
                //   • Single-line only? Allow newlines?
                //   • Caret rendering — blinking |, color-flash on the buffer, or none?
                //   • Modifier handling (shift/ctrl) — ignore or honor?
                //
                demos
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|cell| {
                        cell.spawn((
                            Text::new("Text input + keyboard events"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor::default(),
                            TextRole::Primary,
                        ));
                        cell.spawn((
                            Text::new(
                                "Click to focus, type to fill the buffer, Backspace to \
                                 delete, Escape to unfocus. Bevy ships no text-editing \
                                 primitives — this is hand-rolled from KeyboardInput events.",
                            ),
                            TextColor::default(),
                            TextRole::Subtle,
                        ));
                        cell.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(12.0),
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(12.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor::default(),
                            BgRole::Surface,
                        ))
                        .with_children(|row| {
                            // Field: clickable bordered box with a child Text.
                            row.spawn((
                                Button,
                                Node {
                                    width: Val::Px(280.0),
                                    height: Val::Px(32.0),
                                    align_items: AlignItems::Center,
                                    padding: UiRect::axes(Val::Px(8.0), Val::Px(0.0)),
                                    border: UiRect::all(Val::Px(2.0)),
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor::default(),
                                BgRole::Input,
                                BorderColor::default(),
                                TextInput,
                                TextInputBuffer::default(),
                            ))
                            .with_child((
                                Text::new("(click and type)"),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor::default(),
                                TextRole::Subtle,
                                TextInputDisplay,
                            ));

                            // Emoji-menu toggle button (smile-o icon).
                            row.spawn((
                                Button,
                                Node {
                                    width: Val::Px(32.0),
                                    height: Val::Px(32.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor::default(),
                                BgRole::EmojiBtnIdle,
                                EmojiMenuButton,
                            ))
                            .with_child((
                                Text::new("\u{f118}"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor::default(),
                                TextRole::Primary,
                            ));

                            // Popup menu — absolutely positioned just below the row,
                            // hidden until the toggle button is clicked.
                            row.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    top: Val::Percent(100.0),
                                    right: Val::Px(0.0),
                                    margin: UiRect::top(Val::Px(4.0)),
                                    padding: UiRect::all(Val::Px(8.0)),
                                    column_gap: Val::Px(4.0),
                                    row_gap: Val::Px(4.0),
                                    flex_wrap: FlexWrap::Wrap,
                                    width: Val::Px(160.0), // 4 items × ~36px + gaps
                                    border_radius: BorderRadius::all(Val::Px(6.0)),
                                    display: Display::None, // start hidden
                                    ..default()
                                },
                                BackgroundColor::default(),
                                BgRole::Surface,
                                EmojiMenu,
                            ))
                            .with_children(|menu| {
                                for ch in EMOJI_ICONS {
                                    menu.spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(32.0),
                                            height: Val::Px(32.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border_radius: BorderRadius::all(Val::Px(4.0)),
                                            ..default()
                                        },
                                        BackgroundColor::default(),
                                        BgRole::EmojiBtnIdle,
                                        EmojiMenuItem(ch),
                                    ))
                                    .with_child((
                                        Text::new(ch.to_string()),
                                        TextFont {
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor::default(),
                                        TextRole::Primary,
                                    ));
                                }
                            });
                        });
                    });
            });
        })
        .id()
}

/// Every frame, color the button by its current Interaction state. Same idea as
/// `nav::update_tab_visuals`, just specialised to widgets-section buttons.
#[allow(clippy::type_complexity)]
pub fn update_button_visuals(
    theme: Res<Theme>,
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<ClickCount>),
    >,
) {
    for (interaction, mut bg) in &mut buttons {
        bg.0 = match *interaction {
            Interaction::Pressed => theme.bg.button_pressed,
            Interaction::Hovered => theme.bg.button_hover,
            Interaction::None => theme.bg.button_idle,
        };
    }
}

/// Bumps `ClickCount` on `Pressed` and rewrites the child `ClickCountLabel`'s
/// text. Demonstrates the "button entity owns its state, child label reflects it"
/// split that's idiomatic in Bevy widget design.
pub fn update_click_buttons(
    mut buttons: Query<(&Interaction, &Children, &mut ClickCount), Changed<Interaction>>,
    mut labels: Query<&mut Text, With<ClickCountLabel>>,
) {
    for (interaction, children, mut count) in &mut buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        count.0 += 1;
        for child in children.iter() {
            if let Ok(mut text) = labels.get_mut(child) {
                text.0 = format!("Clicked: {}", count.0);
            }
        }
    }
}

/// On `Interaction::Pressed`, flip the `Checked` bool on each checkbox and
/// rewrite the child `CheckboxGlyph` Text to "\u{f00c}" (Nerd Font fa-check)
/// when checked, or "" when unchecked.
pub fn update_checkboxes(
    mut checkboxes: Query<(&Interaction, &Children, &mut Checked), Changed<Interaction>>,
    mut glyphs: Query<&mut Text, With<CheckboxGlyph>>,
) {
    for (interaction, children, mut checked) in &mut checkboxes {
        if *interaction != Interaction::Pressed {
            continue;
        }
        checked.0 = !checked.0;
        for child in children.iter() {
            if let Ok(mut text) = glyphs.get_mut(child) {
                text.0 = if checked.0 { "\u{f00c}".into() } else { "".into() };
            }
        }
    }
}

/// While a slider track is `Interaction::Pressed`, read the cursor's x position
/// from the window each frame and update `SliderValue` in 0.0..=1.0.
///
/// Bevy 0.18 split UI transforms off into `UiGlobalTransform` (a `Deref`-wrapped
/// `Affine2`). Querying the standard `&GlobalTransform` returns no UI entities
/// — that was the bug.
///
/// Coordinate-space note: `ComputedNode.size()` and `UiGlobalTransform.translation`
/// are in *physical* pixels. The default `cursor_position()` returns *logical*
/// pixels (physical ÷ scale_factor), so on HiDPI displays the math would be off
/// — we use `physical_cursor_position()` to keep units consistent.
pub fn update_slider_drag(
    window: Single<&Window>,
    mut sliders: Query<
        (&Interaction, &ComputedNode, &UiGlobalTransform, &mut SliderValue),
        With<Slider>,
    >,
) {
    let Some(cursor) = window.physical_cursor_position() else {
        return;
    };
    for (interaction, computed, transform, mut value) in &mut sliders {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let track_width = computed.size().x;
        if track_width <= 0.0 {
            continue;
        }
        // `UiGlobalTransform` derefs to `Affine2`, whose `translation: Vec2`
        // is the screen-pixel position of the node's center.
        let track_left = transform.translation.x - track_width / 2.0;
        let local_x = cursor.x - track_left;
        value.0 = (local_x / track_width).clamp(0.0, 1.0);
    }
}

/// When `SliderValue` changes (drag, or any other code that writes it),
/// reposition the child thumb so its `left` is `value * 100%` of the track
/// width. Runs only on entities whose value changed this frame.
pub fn position_slider_thumbs(
    tracks: Query<(&SliderValue, &Children), Changed<SliderValue>>,
    mut thumbs: Query<&mut Node, With<SliderThumb>>,
) {
    for (value, children) in &tracks {
        for child in children.iter() {
            if let Ok(mut node) = thumbs.get_mut(child) {
                node.left = Val::Percent(value.0 * 100.0);
            }
        }
    }
}

/// Rewrite each `SliderValueLabel`'s text whenever the slider it points at
/// changes value. Cross-entity reference: the label holds an `Entity` and looks
/// up the track in a separate query.
///
/// The `Changed<SliderValue>` filter on `sliders` means `sliders.get(...)` only
/// returns Ok for tracks whose value was written this frame — so labels for
/// idle sliders naturally skip the rewrite.
pub fn update_slider_value_labels(
    sliders: Query<&SliderValue, Changed<SliderValue>>,
    mut labels: Query<(&SliderValueLabel, &mut Text)>,
) {
    if sliders.is_empty() {
        return;
    }
    for (label, mut text) in &mut labels {
        if let Ok(value) = sliders.get(label.slider) {
            text.0 = format!("value: {:.2}", value.0);
        }
    }
}

/// On startup, give the Window a `CursorIcon` component so other systems can
/// mutate it. Bevy doesn't add this by default — without it, `Single<&mut
/// CursorIcon>` would panic with "no matching entities".
pub fn init_cursor_icon(mut commands: Commands, window: Single<Entity, With<Window>>) {
    commands
        .entity(*window)
        .insert(CursorIcon::from(SystemCursorIcon::Default));
}

/// While the cursor is over (or pressing) any `Slider`, set the window cursor
/// to `Pointer`. When it leaves, restore `Default`. Uses `Changed<Interaction>`
/// so the system only fires on hover-state transitions, not every frame.
pub fn update_cursor_for_sliders(
    sliders: Query<&Interaction, (With<Slider>, Changed<Interaction>)>,
    mut cursor: Single<&mut CursorIcon>,
) {
    for interaction in &sliders {
        **cursor = match *interaction {
            Interaction::Hovered | Interaction::Pressed => {
                CursorIcon::from(SystemCursorIcon::Pointer)
            }
            Interaction::None => CursorIcon::from(SystemCursorIcon::Default),
        };
    }
}

/// On `Pressed` of a TextInput, set it as the focused field in the
/// `FocusedTextInput` Resource. We use Pressed (not Hovered) so just hovering
/// doesn't steal focus.
#[allow(clippy::type_complexity)]
pub fn update_text_input_focus(
    inputs: Query<(Entity, &Interaction), (Changed<Interaction>, With<TextInput>)>,
    mut focused: ResMut<FocusedTextInput>,
) {
    for (entity, interaction) in &inputs {
        if *interaction == Interaction::Pressed {
            focused.0 = Some(entity);
        }
    }
}

/// Read keyboard events while a field is focused and mutate its buffer.
///
/// Strategy:
///   • `event.text` is `Option<SmolStr>` — already accounts for shift, dead
///     keys, and keyboard layout. If present, append (filtering control chars).
///   • For Backspace and Escape, match on `event.key_code` regardless of layout.
///
/// We listen only to `state == Pressed` events to avoid double-applying on
/// release. `repeat: true` events are kept so holding a key works.
pub fn update_text_input_keyboard(
    mut events: MessageReader<KeyboardInput>,
    mut focused: ResMut<FocusedTextInput>,
    mut buffers: Query<&mut TextInputBuffer>,
) {
    let Some(focused_entity) = focused.0 else {
        events.clear();
        return;
    };
    let Ok(mut buffer) = buffers.get_mut(focused_entity) else {
        // Focused entity doesn't exist anymore — clear focus.
        focused.0 = None;
        events.clear();
        return;
    };

    for event in events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match event.key_code {
            KeyCode::Escape => {
                focused.0 = None;
                return;
            }
            KeyCode::Backspace => {
                buffer.0.pop();
            }
            _ => {
                if let Some(text) = &event.text {
                    for ch in text.chars() {
                        // Filter control chars (newline, tab, etc) and respect cap.
                        if !ch.is_control() && buffer.0.chars().count() < TEXT_INPUT_MAX_LEN {
                            buffer.0.push(ch);
                        }
                    }
                }
            }
        }
    }
}

/// Rewrite each text input's child Text. Runs every frame — only a few inputs
/// in this app, so the cost is negligible. Could be filtered with
/// `Changed<TextInputBuffer>` to skip work when nothing changed, but then we'd
/// also need a path that runs on focus changes. Simplicity wins here.
pub fn update_text_input_display(
    theme: Res<Theme>,
    focused: Res<FocusedTextInput>,
    inputs: Query<(Entity, &TextInputBuffer, &Children), With<TextInput>>,
    mut displays: Query<(&mut Text, &mut TextColor), With<TextInputDisplay>>,
) {
    for (entity, buffer, children) in &inputs {
        let is_focused = focused.0 == Some(entity);
        for child in children.iter() {
            if let Ok((mut text, mut color)) = displays.get_mut(child) {
                if buffer.0.is_empty() && !is_focused {
                    text.0 = "(click and type)".to_string();
                    color.0 = theme.text.subtle;
                } else {
                    text.0 = if is_focused {
                        format!("{}|", buffer.0)
                    } else {
                        buffer.0.clone()
                    };
                    color.0 = theme.text.primary;
                }
            }
        }
    }
}

/// Color the field's border based on focus state — visible feedback that
/// keystrokes will land here.
///
/// Owns the input border end-to-end: no `BorderRole` on the entity. Runs
/// unconditionally so the focus/subtle color stays correct through theme
/// transitions (blends both sides), focus changes, and the trailing edge of
/// a transition (where neither focus nor the Theme resource fires `Changed`,
/// but the previous frame painted a mid-blend value). Cost: one BorderColor
/// write per input per frame — trivial at this entity count.
pub fn update_text_input_border(
    theme: Res<Theme>,
    transition: Option<Res<ThemeTransition>>,
    focused: Res<FocusedTextInput>,
    mut inputs: Query<(Entity, &mut BorderColor), With<TextInput>>,
) {
    use bevy::color::Mix;
    let (focus_color, subtle_color) = if let Some(t) = transition.as_ref() {
        let p = t.eased_progress();
        (
            t.from_border.focus.mix(&theme.border.focus, p),
            t.from_border.subtle.mix(&theme.border.subtle, p),
        )
    } else {
        (theme.border.focus, theme.border.subtle)
    };
    for (entity, mut border) in &mut inputs {
        let color = if focused.0 == Some(entity) {
            focus_color
        } else {
            subtle_color
        };
        *border = BorderColor::all(color);
    }
}

/// On Pressed of the emoji-menu button, flip every `EmojiMenu`'s display
/// between `None` and `Flex` to show/hide.
pub fn toggle_emoji_menu(
    button: Query<&Interaction, (Changed<Interaction>, With<EmojiMenuButton>)>,
    mut menus: Query<&mut Node, With<EmojiMenu>>,
) {
    for interaction in &button {
        if *interaction != Interaction::Pressed {
            continue;
        }
        for mut node in &mut menus {
            node.display = match node.display {
                Display::None => Display::Flex,
                _ => Display::None,
            };
        }
    }
}

/// On Pressed of an `EmojiMenuItem`, append its character to the buffer of
/// whatever text input is currently focused. Then hide the menu.
pub fn handle_emoji_clicks(
    items: Query<(&Interaction, &EmojiMenuItem), Changed<Interaction>>,
    focused: Res<FocusedTextInput>,
    mut buffers: Query<&mut TextInputBuffer>,
    mut menus: Query<&mut Node, With<EmojiMenu>>,
) {
    for (interaction, item) in &items {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Some(target) = focused.0
            && let Ok(mut buffer) = buffers.get_mut(target)
            && buffer.0.chars().count() < TEXT_INPUT_MAX_LEN
        {
            buffer.0.push(item.0);
        }
        // Auto-close after a pick — common UX expectation.
        for mut menu_node in &mut menus {
            menu_node.display = Display::None;
        }
    }
}

/// Hover/press feedback for the emoji menu button and items — same idea as
/// the click-counter button, just specialized to these widgets.
#[allow(clippy::type_complexity)]
pub fn update_emoji_button_visuals(
    theme: Res<Theme>,
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<Button>,
            Or<(With<EmojiMenuButton>, With<EmojiMenuItem>)>,
        ),
    >,
) {
    for (interaction, mut bg) in &mut buttons {
        bg.0 = match *interaction {
            Interaction::Hovered | Interaction::Pressed => theme.bg.emoji_btn_hover,
            Interaction::None => theme.bg.emoji_btn_idle,
        };
    }
}

/// Insert a `Tween<f32>` on `UiTransform.scale` whenever a `ClickCount`
/// button's `Interaction` changes. Reads the current scale (mid-tween or
/// not) so a fast re-hover continues smoothly from the visible position
/// rather than snapping to 1.0.
///
/// Tab buttons / slider tracks / emoji buttons are NOT targeted (filter
/// uses `With<ClickCount>`), so chrome stays instant.
#[allow(clippy::type_complexity)]
pub fn dispatch_button_hover_scale(
    mut commands: Commands,
    q: Query<
        (Entity, &Interaction, &UiTransform),
        (Changed<Interaction>, With<ClickCount>),
    >,
) {
    for (entity, interaction, transform) in &q {
        let current = transform.scale.x;
        let (target, duration) = match *interaction {
            Interaction::Hovered => (1.05, 0.15),
            Interaction::None => (1.0, 0.15),
            Interaction::Pressed => (0.97, 0.08),
        };
        commands.entity(entity).insert(Tween::<f32> {
            start: current,
            end: target,
            elapsed: 0.0,
            duration,
            easing: EaseFunction::QuadraticOut,
        });
    }
}
