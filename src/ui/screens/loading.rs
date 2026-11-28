use bevy::prelude::*;
use delta_slayers_ui::CarbonTheme;
use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hash, Hasher},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::app::state::AppState;

const LOADING_CAPTIONS: [&str; 6] = [
    "Loading the revolution… Marx is taking his sweet time.",
    "The workers of the world are reuniting. Please wait.",
    "Rebooting communism…",
    "Marx.exe has stopped responding. Obama is on it.",
    "From each according to his loading speed, to each according to his patience.",
    "Resurrecting the class struggle…",
];

#[derive(Component, Clone, Copy, Default)]
pub(in crate::ui) struct LoadingBarSegment;

/// Shown while gameplay loads the Blender-authored world.
pub(in crate::ui) fn scene() -> impl SceneList {
    scene_for(AppState::Loading)
}

/// Temporary standalone preview; unlike AppState::Loading, this does not start gameplay.
pub(in crate::ui) fn preview_scene() -> impl SceneList {
    scene_for(AppState::LoadingPreview)
}

fn scene_for(state: AppState) -> impl SceneList {
    bsn_list! {
        Camera2d
        Camera {
            order: 10,
        }
        IsDefaultUiCamera
        DespawnOnExit::<AppState>(state)
        --
        @loading_content()
    }
}

fn loading_content() -> impl Scene {
    let caption = random_loading_caption();

    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            min_width: px(0),
            min_height: px(0),
            padding: UiRect::axes(px(20), px(24)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: px(18),
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().page)))
        Children [
            // Keep the source image's 640:542 aspect ratio inside a framed viewport.
            Node {
                width: Val::Vw(54.0),
                max_width: px(760),
                max_height: Val::Vh(56.0),
                min_width: px(240),
                aspect_ratio: Option::Some(1.181),
                padding: UiRect::all(px(8)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(px(10)),
                overflow: Overflow::clip(),
            }
            template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().surface_deep)))
            Children [
                Node {
                    width: percent(100),
                    height: percent(100),
                    min_width: px(0),
                    min_height: px(0),
                }
                template(|ctx| {
                    let image = ctx.resource::<AssetServer>().load("ui/loading.png");
                    Ok(ImageNode::new(image).with_mode(NodeImageMode::Stretch))
                })
            ]
            --
            Node {
                width: percent(92),
                max_width: px(760),
                align_items: AlignItems::Center,
            }
            Children [
                Text(caption)
                TextFont {
                    font_size: px(24),
                }
                template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().ink)))
                TextLayout::justify(Justify::Center)
                --
                Node {
                    width: px(280),
                    height: px(7),
                    margin: UiRect::top(px(14)),
                    border_radius: BorderRadius::all(px(4)),
                    overflow: Overflow::clip(),
                }
                template(|_| Ok(BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.22))))
                Children [
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(0),
                        top: px(0),
                        bottom: px(0),
                        width: percent(30),
                        border_radius: BorderRadius::all(px(4)),
                    }
                    LoadingBarSegment
                    template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().accent)))
                ]
            ]
        ]
    }
}

/// Runs for both the actual loading screen and the standalone preview.
pub(in crate::ui) fn loading_screen_active(state: Res<State<AppState>>) -> bool {
    matches!(*state.get(), AppState::Loading | AppState::LoadingPreview)
}

/// Moves an indeterminate segment across the loading track.
pub(in crate::ui) fn animate_loading_bar(
    time: Res<Time>,
    mut phase: Local<f32>,
    mut segments: Query<&mut Node, With<LoadingBarSegment>>,
) {
    *phase = (*phase + time.delta_secs() * 0.8) % 2.0;
    let progress = if *phase <= 1.0 {
        *phase
    } else {
        2.0 - *phase
    };

    for mut node in &mut segments {
        node.left = percent(progress * 70.0);
    }
}

fn random_loading_caption() -> &'static str {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let mut hasher = RandomState::new().build_hasher();
    time.as_nanos().hash(&mut hasher);

    LOADING_CAPTIONS[hasher.finish() as usize % LOADING_CAPTIONS.len()]
}
