use bevy::{prelude::*, ui_widgets::Activate};
use delta_slayers_ui::{
    CarbonButtonTone, CarbonIcon, CarbonTheme, action_button, muted_text, page_title, panel,
};

use crate::app::state::AppState;

pub(in crate::ui) fn continue_screen() -> impl SceneList {
    placeholder_screen(
        AppState::Continue,
        "Continue",
        "Continuing a saved session is not implemented yet.",
    )
}

pub(in crate::ui) fn load_screen() -> impl SceneList {
    placeholder_screen(
        AppState::Load,
        "Load game",
        "Saved-game selection and loading are not connected yet.",
    )
}

pub(in crate::ui) fn settings_screen() -> impl SceneList {
    placeholder_screen(
        AppState::Settings,
        "Settings",
        "Application and gameplay settings will be available here.",
    )
}

pub(in crate::ui) fn credits_screen() -> impl SceneList {
    placeholder_screen(
        AppState::Credits,
        "Credits",
        "Project credits and acknowledgements will be listed here.",
    )
}

fn placeholder_screen(
    state: AppState,
    title: &'static str,
    description: &'static str,
) -> impl SceneList {
    bsn_list! {
        Camera2d
        DespawnOnExit::<AppState>(state)
        --
        @placeholder_content(title, description, state)
    }
}

fn placeholder_content(
    title: &'static str,
    description: &'static str,
    state: AppState,
) -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(12)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().page)))
        DespawnOnExit::<AppState>(state)
        Children [
            Node {
                width: px(480),
                max_width: percent(92),
                max_height: percent(90),
                min_width: px(0),
                overflow: Overflow::scroll(),
            }
            Children [
                @panel(percent(100))
                Children [
                    @page_title(title)
                    --
                    @muted_text(description, 13.0)
                    --
                    @action_button("Back to main menu", CarbonIcon::Back, CarbonButtonTone::Secondary)
                    on(|_activate: On<Activate>, mut next: ResMut<NextState<AppState>>| {
                        next.set(AppState::MainMenu);
                    })
                ]
            ]
        ]
    }
}
