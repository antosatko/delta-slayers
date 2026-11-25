use bevy::{prelude::*, ui_widgets::Activate};
use delta_slayers_ui::{
    CarbonButtonTone, CarbonIcon, CarbonTheme, action_button, muted_text, page_title, panel,
    section_header,
};

use crate::{
    app::state::AppState,
    gameplay::player::PlayerSettings,
    ui::components::PlayerSettingControl,
};

use super::gameplay::{player_setting_control, player_setting_value_changed};

pub(in crate::ui) fn scene() -> impl SceneList {
    bsn_list! {
        Camera2d
        DespawnOnExit::<AppState>(AppState::Settings)
        --
        @settings_screen()
    }
}

fn settings_screen() -> impl Scene {
    let defaults = PlayerSettings::default();

    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(16)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().page)))
        DespawnOnExit::<AppState>(AppState::Settings)
        Children [
            Node {
                width: px(620),
                max_width: percent(96),
                max_height: percent(94),
                min_width: px(0),
                overflow: Overflow::scroll(),
            }
            Children [
                @panel(percent(100))
                Children [
                    @page_title("Settings")
                    --
                    @muted_text("Changes apply immediately.", 12.0)
                    --
                    @action_button("Restore defaults", CarbonIcon::Back, CarbonButtonTone::Ghost)
                    on(|_activate: On<Activate>, mut settings: ResMut<PlayerSettings>| {
                        *settings = default();
                    })
                    --
                    @section_header(CarbonIcon::Settings, "Camera")
                    --
                    @player_setting_control(
                        "Pan sensitivity",
                        "Middle-mouse drag speed.",
                        PlayerSettingControl::PanSensitivity,
                        defaults.camera_pan_sensitivity,
                        0.0002,
                        0.01,
                        0.0001,
                        4
                    )
                    --
                    @player_setting_control(
                        "Zoom sensitivity",
                        "Scroll-wheel zoom per line.",
                        PlayerSettingControl::ZoomSensitivity,
                        defaults.camera_zoom_sensitivity,
                        0.1,
                        2.0,
                        0.05,
                        2
                    )
                    --
                    @player_setting_control(
                        "Minimum camera distance",
                        "Closest allowed follow-camera distance.",
                        PlayerSettingControl::MinCameraDistance,
                        defaults.camera_min_distance,
                        4.0,
                        30.0,
                        0.5,
                        1
                    )
                    --
                    @player_setting_control(
                        "Maximum camera distance",
                        "Farthest allowed follow-camera distance.",
                        PlayerSettingControl::MaxCameraDistance,
                        defaults.camera_max_distance,
                        8.0,
                        60.0,
                        0.5,
                        1
                    )
                    --
                    @section_header(CarbonIcon::Character, "Movement")
                    --
                    @player_setting_control(
                        "Walk speed",
                        "Normal keyboard and click-to-move speed.",
                        PlayerSettingControl::WalkSpeed,
                        defaults.walk_speed,
                        1.0,
                        12.0,
                        0.25,
                        2
                    )
                    --
                    @player_setting_control(
                        "Sprint speed",
                        "Speed while holding Shift.",
                        PlayerSettingControl::SprintSpeed,
                        defaults.sprint_speed,
                        1.0,
                        20.0,
                        0.25,
                        2
                    )
                    --
                    @action_button("Back to main menu", CarbonIcon::Back, CarbonButtonTone::Primary)
                    on(|_activate: On<Activate>, mut next: ResMut<NextState<AppState>>| {
                        next.set(AppState::MainMenu);
                    })
                ]
            ]
        ]
    }
}
