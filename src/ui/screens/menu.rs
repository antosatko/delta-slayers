use bevy::{prelude::*, ui_widgets::Activate};
use delta_slayers_ui::{
    SlateButtonTone, SlateIcon, SlateTheme, action_button, page_header, panel, section_header,
};

use crate::app::state::AppState;

pub(in crate::ui) fn scene() -> impl SceneList {
    bsn_list! {
        Camera2d
        DespawnOnExit::<AppState>(AppState::MainMenu)
        --
        @main_menu()
    }
}

fn main_menu() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        template(|ctx| {
            Ok(BackgroundColor(ctx.resource::<SlateTheme>().page))
        })
        DespawnOnExit::<AppState>(AppState::MainMenu)
        Children [
            Node {
                width: px(470),
                max_width: percent(92),
                flex_direction: FlexDirection::Column,
                row_gap: px(16),
            }
            Children [
                @page_header(
                    "GREYBRIDGE CHAPTER",
                    "Delta Slayers",
                    "A guild contract awaits."
                )
                --
                @panel(percent(100))
                Children [
                    @section_header(SlateIcon::Compass, "Continue your journey")
                    --
                    @menu_button("Play", SlateIcon::Sword, AppState::Loading, SlateButtonTone::Primary)
                    --
                    @menu_button("Continue", SlateIcon::Back, AppState::Continue, SlateButtonTone::Secondary)
                    --
                    @menu_button("Load game", SlateIcon::Inventory, AppState::Load, SlateButtonTone::Secondary)
                    --
                    @menu_button("Settings", SlateIcon::Settings, AppState::Settings, SlateButtonTone::Secondary)
                    --
                    @menu_button("Credits", SlateIcon::Book, AppState::Credits, SlateButtonTone::Ghost)
                    --
                    @exit_button()
                ]
            ]
        ]
    }
}

fn menu_button(
    label: &'static str,
    icon: SlateIcon,
    target: AppState,
    tone: SlateButtonTone,
) -> impl Scene {
    bsn! {
        @action_button(label, icon, tone)
        on(move |_activate: On<Activate>, mut next_state: ResMut<NextState<AppState>>| {
            info!("Transition: Moving to {:?}", target);
            next_state.set(target);
        })
    }
}

fn exit_button() -> impl Scene {
    bsn! {
        @action_button("Exit", SlateIcon::Close, SlateButtonTone::Danger)
        on(|_activate: On<Activate>, mut app_exit: MessageWriter<AppExit>| {
            app_exit.write(AppExit::Success);
        })
    }
}
