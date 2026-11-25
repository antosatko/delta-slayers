use bevy::prelude::*;
use delta_slayers_ui::{
    CarbonIcon, CarbonStatus, CarbonTheme, badge, eyebrow_text, muted_text, page_header, panel,
    section_header,
};

use crate::app::state::AppState;

/// Shown while gameplay loads the Blender-authored world.
pub(in crate::ui) fn scene() -> impl SceneList {
    bsn_list! {
        Camera2d
        Camera {
            order: 10,
        }
        IsDefaultUiCamera
        DespawnOnExit::<AppState>(AppState::Loading)
        --
        @loading_content()
    }
}

fn loading_content() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(24)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().page)))
        DespawnOnExit::<AppState>(AppState::Loading)
        Children [
            Node {
                width: px(440),
                max_width: percent(92),
                min_width: px(0),
            }
            Children [
                @page_header(
                    "Delta Slayers",
                    "Loading",
                    "The town and its authored gameplay objects are being prepared."
                )
                --
                @panel(percent(100))
                Children [
                    @section_header(CarbonIcon::Map, "Preparing the world")
                    --
                    @badge("LOADING", CarbonStatus::Warning)
                    --
                    @eyebrow_text("WORLD INITIALIZATION")
                    --
                    @muted_text("Preparing the town...", 13.0)
                    --
                    Node {
                        width: percent(100),
                        height: px(5),
                        border_radius: BorderRadius::all(px(3)),
                        overflow: Overflow::clip(),
                    }
                    template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().surface_deep)))
                    Children [
                        Node {
                            width: percent(42),
                            height: percent(100),
                            border_radius: BorderRadius::all(px(3)),
                        }
                        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().accent)))
                    ]
                ]
            ]
        ]
    }
}
