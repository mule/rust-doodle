use bevy::prelude::*;
use bevy::text::Font;

mod animations_section;
mod layout_section;
mod nav;
mod theme;
mod theming_section;
mod tween;
mod widgets_section;

use nav::Section;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "bevy-ui-showcase".to_string(),
                // Sensible default if the maximize request fails or before it
                // takes effect — large enough to fit all four sections without
                // forcing the user to resize.
                resolution: (1400u32, 900u32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(theme::Theme::default())
        .insert_resource(ClearColor(theme::Theme::default().bg.background))
        .init_resource::<nav::CurrentSection>()
        .init_resource::<widgets_section::FocusedTextInput>()
        .add_systems(
            Startup,
            (
                override_default_font,
                widgets_section::init_cursor_icon,
                maximize_window,
                setup_root,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                nav::handle_tab_clicks,
                nav::update_section_visibility,
                nav::update_tab_visuals,
                layout_section::update_percent_labels,
                widgets_section::update_button_visuals,
                widgets_section::update_click_buttons,
                widgets_section::update_checkboxes,
                widgets_section::update_slider_drag,
                widgets_section::position_slider_thumbs,
                widgets_section::update_slider_value_labels,
                widgets_section::update_cursor_for_sliders,
                widgets_section::update_text_input_focus,
                widgets_section::update_text_input_keyboard,
                widgets_section::update_text_input_display,
                widgets_section::update_text_input_border,
                widgets_section::toggle_emoji_menu,
                widgets_section::handle_emoji_clicks,
                widgets_section::update_emoji_button_visuals,
                widgets_section::dispatch_button_hover_scale,
            ),
        )
        .add_systems(
            Update,
            (
                theme::resolve_bg_role,
                theme::resolve_text_role,
                theme::resolve_border_role,
                theme::sync_clear_color,
                theme::handle_theme_toggle,
                nav::update_theme_toggle_label,
                theming_section::update_swatches,
                theme::advance_theme_transition,
                tween::advance_f32_tweens,
                tween::advance_color_tweens,
                tween::advance_val_tweens,
                animations_section::toggle_drawer,
            ),
        )
        .run();
}

/// Replace Bevy's default font (FiraMono-subset, no extended glyphs) with
/// HackNerdFont so ✓, em-dashes, arrows, and ~9k Nerd Font icons render across
/// every Text in the app without per-widget font handles.
///
/// We embed the bytes via `include_bytes!` rather than loading via AssetServer
/// because we need the swap to happen synchronously in Startup, before any
/// other Startup system spawns text. AssetServer loads are async — text spawned
/// the same frame would still flash with the old default.
fn override_default_font(mut fonts: ResMut<Assets<Font>>) {
    const FONT_BYTES: &[u8] =
        include_bytes!("../assets/fonts/HackNerdFont-Regular.ttf");
    let font = Font::try_from_bytes(FONT_BYTES.to_vec())
        .expect("failed to parse HackNerdFont-Regular.ttf");
    // `insert` returns Result for index-based IDs but never errors for the
    // UUID-based default handle (per Assets::insert docs).
    fonts
        .insert(&Handle::<Font>::default(), font)
        .expect("default font handle is UUID-based; insert cannot fail");
}

/// Ask the OS to maximize the primary window at startup. We don't auto-fit to
/// content (Bevy's window opens before layout runs), but maximizing means the
/// user gets all available screen space immediately rather than having to
/// drag-resize.
fn maximize_window(mut window: Single<&mut Window>) {
    window.set_maximized(true);
}

fn setup_root(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let root = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .id();

    let tab_bar = nav::spawn_tab_bar(&mut commands);
    let content = commands
        .spawn(Node {
            flex_grow: 1.0,
            width: Val::Percent(100.0),
            ..default()
        })
        .id();
    commands.entity(root).add_children(&[tab_bar, content]);

    let inter_bold: Handle<Font> = asset_server.load("fonts/Inter-Bold.ttf");

    let mut sections = Vec::with_capacity(Section::ALL.len());
    sections.push(layout_section::spawn(&mut commands));
    sections.push(widgets_section::spawn(&mut commands));
    sections.push(theming_section::spawn(&mut commands, inter_bold));
    sections.push(animations_section::spawn(&mut commands));
    commands.entity(content).add_children(&sections);
}
