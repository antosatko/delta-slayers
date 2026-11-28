use bevy::prelude::*;
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
        }
        template(|ctx| {
            let image = ctx.resource::<AssetServer>().load("ui/loading.png");
            Ok(ImageNode::new(image).with_mode(NodeImageMode::Stretch))
        })
        Children [
            Node {
                width: percent(100),
                height: percent(100),
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
            }
            template(|_| Ok(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.16))))
            --
            Node {
                width: percent(92),
                max_width: px(1160),
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                bottom: px(44),
                margin: UiRect::horizontal(auto()),
                padding: UiRect::axes(px(24), px(18)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(8)),
            }
            template(|_| Ok(BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.68))))
            Children [
                Text(caption)
                TextFont {
                    font_size: px(28),
                }
                TextColor(Color::WHITE)
                TextLayout::justify(Justify::Center)
            ]
        ]
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
