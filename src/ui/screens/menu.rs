use bevy::{prelude::*, ui_widgets::Activate};
use delta_slayers_ui::{
    CarbonButtonTone, CarbonIcon, CarbonTheme, action_button, page_header, panel, section_header,
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
            Ok(BackgroundColor(ctx.resource::<CarbonTheme>().page))
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
                    "Delta Slayers",
                    "Guild hall",
                    "Choose whether to begin a new journey, continue an existing one, or adjust the game settings."
                )
                --
                @panel(percent(100))
                Children [
                    @section_header(CarbonIcon::Compass, "Continue your journey")
                    --
                    @menu_button("Play", CarbonIcon::Sword, AppState::Loading, CarbonButtonTone::Primary)
                    --
                    @menu_button("Continue", CarbonIcon::Back, AppState::Continue, CarbonButtonTone::Secondary)
                    --
                    @menu_button("Load game", CarbonIcon::Inventory, AppState::Load, CarbonButtonTone::Secondary)
                    --
                    @menu_button("Settings", CarbonIcon::Settings, AppState::Settings, CarbonButtonTone::Secondary)
                    --
                    @menu_button("Credits", CarbonIcon::Book, AppState::Credits, CarbonButtonTone::Ghost)
                    --
                    @exit_button()
                ]
            ]
        ]
    }
}

fn menu_button(
    label: &'static str,
    icon: CarbonIcon,
    target: AppState,
    tone: CarbonButtonTone,
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
        @action_button("Exit", CarbonIcon::Close, CarbonButtonTone::Danger)
        on(|_activate: On<Activate>, mut app_exit: MessageWriter<AppExit>| {
            app_exit.write(AppExit::Success);
        })
    }
}
