use bevy::{
    app::Propagate,
    feathers::{
        containers::{pane, pane_body, pane_header},
        controls::{
            FeathersCheckbox, FeathersNumberInput, FeathersSlider, HardLimit, NumberInputPrecision,
            NumberInputStep, NumberInputValue,
        },
        display::{label, label_dim},
        theme::{SurfaceLevel, ThemeBackgroundColor, ThemeContext},
        tokens,
    },
    prelude::*,
    ui::{Checked, RelativeCursorPosition},
    ui_widgets::{Activate, SliderPrecision, SliderStep, SliderValue, ValueChange},
};
use delta_slayers_ui::{
    CarbonButtonTone, CarbonIcon, CarbonTheme, action_button, eyebrow_text, gear_slot,
    item_art_tile, key_hint, muted_text, page_header, page_title, panel, section_header,
};

use crate::{
    app::state::AppState,
    gameplay::{
        exploration::{ExplorationPortal, ExplorationTravelRequest, PortalAction, VegetationTuning},
        zones::{FastTravelRequest, ZoneKind},
    },
    ui::{
        components::{
            DeveloperSettingsContentRoot, DeveloperSettingsPanelState, ExplorationDebugInfoText,
            ExplorationDebugPanelRoot, ExplorationDebugPanelState, ExplorationParameterControl,
            ExplorationSetupRoot, ExplorationSetupTileCountText, ExplorationSetupViewModel,
            FastTravelButton, FastTravelMenu, GameMenuModal, MinimapExplorationDot, MinimapZoneDot,
            PanelRoot, PauseOverlay, PlayerSettingControl, PlayerSettingToggle,
            PopupConfirmRequested, PopupRoot,
            SettingsPanelRoot, VegetationLabRoot, VegetationSliderControl, VegetationSupportText,
            VegetationToggle,
        },
        inventory::{CurrencyLabel, InventoryRoot},
        stack::{UiLayer, UiStack},
    },
};

/// All in-game UI is declared as BSN scenes. Gameplay setup must not spawn UI.
pub(in crate::ui) fn scene() -> impl SceneList {
    bsn_list! {
        Camera2d
        Camera {
            order: 1,
        }
        IsDefaultUiCamera
        DespawnOnExit::<AppState>(AppState::InGame)
        --
        @shop_panel()
        --
        @guild_panel()
        --
        @interaction_prompt()
        --
        @fast_travel_menu()
        --
        @minimap()
        --
        @exploration_debug_panel()
        --
        @vegetation_lab_panel()
        --
        @exploration_setup_panel()
        --
        @inventory_scene()
        --
        @player_settings_panel()
        --
        @game_menu_modal()
        --
        @pause_overlay()
    }
}

fn zone_panel(
    kind: ZoneKind,
    title: &'static str,
    icon: CarbonIcon,
    section: &'static str,
    description: &'static str,
) -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            right: px(16),
            width: px(390),
            max_width: percent(100),
            min_width: px(0),
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().overlay)))
        bevy::ui::GlobalZIndex(10)
        PanelRoot(kind)
        RelativeCursorPosition
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            @panel(percent(100))
            Children [
                @page_title(title)
                --
                @section_header(icon, section)
                --
                @eyebrow_text("NOT AVAILABLE YET")
                --
                @muted_text(description, 13.0)
                --
                @key_hint("ESC", "Close this window")
            ]
        ]
    }
}

fn shop_panel() -> impl Scene {
    zone_panel(
        ZoneKind::Shop,
        "Shop",
        CarbonIcon::Trade,
        "Merchant stock",
        "Trading is not connected yet. This panel will hold merchant offers and purchase details.",
    )
}

fn guild_panel() -> impl Scene {
    zone_panel(
        ZoneKind::Guild,
        "Guild",
        CarbonIcon::Contracts,
        "Guild board",
        "Guild contracts and party services will be shown here when those systems are implemented.",
    )
}

fn interaction_prompt() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            bottom: px(28),
            width: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        bevy::ui::GlobalZIndex(8)
        PopupRoot
        RelativeCursorPosition
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Node {
                width: px(330),
                max_width: percent(90),
                min_width: px(0),
                padding: UiRect::all(px(10)),
                border_radius: BorderRadius::all(px(8)),
            }
            template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().panel)))
            Children [
                Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: px(12),
                    row_gap: px(8),
                }
                Children [
                    @key_hint("E", "Interact")
                    --
                    @action_button("Use", CarbonIcon::Confirm, CarbonButtonTone::Primary)
                    on(|_activate: On<Activate>, mut request: ResMut<PopupConfirmRequested>| {
                        request.0 = true;
                    })
                ]
            ]
        ]
    }
}

fn fast_travel_menu() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            right: px(12),
            width: px(224),
            max_width: percent(60),
            min_width: px(0),
        }
        bevy::ui::GlobalZIndex(18)
        FastTravelMenu
        RelativeCursorPosition
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            @panel(percent(100))
            Children [
                @section_header(CarbonIcon::Map, "Fast travel")
                --
                @muted_text("Travel to places you have already visited.", 11.0)
                --
                @fast_travel_button(ZoneKind::Shop, "Shop", CarbonIcon::Trade)
                --
                @fast_travel_button(ZoneKind::Guild, "Guild", CarbonIcon::Contracts)
            ]
        ]
    }
}

fn fast_travel_button(kind: ZoneKind, label: &'static str, icon: CarbonIcon) -> impl Scene {
    bsn! {
        @action_button(label, icon, CarbonButtonTone::Secondary)
        FastTravelButton(kind)
        Visibility::Hidden
        on(move |_activate: On<Activate>, mut request: ResMut<FastTravelRequest>| {
            request.0 = Some(kind);
        })
    }
}

fn exploration_debug_panel() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(14),
            left: px(14),
            width: px(330),
            max_width: percent(90),
            min_width: px(0),
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().overlay)))
        bevy::ui::GlobalZIndex(16)
        ExplorationDebugPanelRoot
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            @panel(percent(100))
            Children [
                @section_header(CarbonIcon::Map, "Exploration settings (DEV)")
                --
                Text("Generated map parameters")
                ExplorationDebugInfoText
                TextFont { font_size: px(12) }
                template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
                --
                @action_button("Vegetation lab · F4", CarbonIcon::Settings, CarbonButtonTone::Primary)
                on(|_activate: On<Activate>, mut ui: ResMut<UiStack>| {
                    ui.push(UiLayer::VegetationLab);
                })
                --
                @action_button("Hide details", CarbonIcon::Close, CarbonButtonTone::Secondary)
                on(|_activate: On<Activate>, mut state: ResMut<ExplorationDebugPanelState>| {
                    state.visible = false;
                })
            ]
        ]
    }
}

/// Live biome, density, blade and wind controls for the current exploration field.
fn vegetation_lab_panel() -> impl Scene {
    let defaults = VegetationTuning::default();
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(14),
            right: px(14),
            width: px(430),
            max_width: percent(46),
            max_height: percent(94),
            min_width: px(0),
            overflow: Overflow::scroll(),
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().overlay)))
        bevy::ui::GlobalZIndex(24)
        VegetationLabRoot
        RelativeCursorPosition
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            @panel(percent(100))
            Children [
                Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: px(8),
                }
                Children [
                    @page_title("Vegetation lab")
                    --
                    @action_button("Close · F4", CarbonIcon::Close, CarbonButtonTone::Ghost)
                    on(|_activate: On<Activate>, mut ui: ResMut<UiStack>| {
                        ui.remove(UiLayer::VegetationLab);
                    })
                ]
                --
                @muted_text("Tune the living surface in real time. The field blends four biome palettes across the map.", 12.0)
                --
                Node {
                    width: percent(100),
                    padding: UiRect::all(px(9)),
                    border_radius: BorderRadius::all(px(6)),
                }
                template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().panel_alt)))
                Children [
                    Text("Checking GPU grass support…")
                    VegetationSupportText
                    TextFont { font_size: px(11) }
                    template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
                ]
                --
                @section_header(CarbonIcon::Settings, "Field quality")
                --
                @vegetation_slider_control("Overall density", "Blade coverage across all biomes.", VegetationSliderControl::GlobalDensity, defaults.global_density, 0.0, 1.4, 0.05, 2)
                --
                @vegetation_slider_control("View distance", "How far grass reaches from the camera (metres).", VegetationSliderControl::ViewDistance, defaults.view_distance, 18.0, 52.0, 1.0, 0)
                --
                @vegetation_slider_control("Blade height", "Scale every plant without changing the ground.", VegetationSliderControl::BladeHeight, defaults.blade_height, 0.35, 1.6, 0.05, 2)
                --
                @vegetation_shadow_toggle()
                --
                @section_header(CarbonIcon::Map, "Biome density")
                --
                @vegetation_slider_control("Grassland", "Wild meadow and flowering grasses.", VegetationSliderControl::GrasslandDensity, defaults.grassland_density, 0.0, 1.8, 0.05, 2)
                --
                @vegetation_slider_control("Woodland", "Mossy forest floor and fern undergrowth.", VegetationSliderControl::WoodlandDensity, defaults.woodland_density, 0.0, 1.8, 0.05, 2)
                --
                @vegetation_slider_control("Dryland", "Sparse, wind-shaped dune grasses.", VegetationSliderControl::DrylandDensity, defaults.dryland_density, 0.0, 1.8, 0.05, 2)
                --
                @vegetation_slider_control("Highlands", "Pale, hardy high-altitude plants.", VegetationSliderControl::HighlandDensity, defaults.highland_density, 0.0, 1.8, 0.05, 2)
                --
                @section_header(CarbonIcon::Settings, "Atmosphere")
                --
                @vegetation_slider_control("Wind strength", "How strongly the blades lean and sway.", VegetationSliderControl::WindStrength, defaults.wind_strength, 0.0, 1.0, 0.05, 2)
                --
                @vegetation_slider_control("Gust speed", "How quickly wind waves travel across the field.", VegetationSliderControl::WindSpeed, defaults.wind_speed, 0.0, 12.0, 0.25, 2)
                --
                Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    column_gap: px(8),
                }
                Children [
                    @action_button("Reset tuning", CarbonIcon::Back, CarbonButtonTone::Secondary)
                    on(|_activate: On<Activate>, mut tuning: ResMut<VegetationTuning>| {
                        *tuning = default();
                    })
                    --
                    @muted_text("F4 · close panel", 11.0)
                ]
            ]
        ]
    }
}

fn vegetation_slider_control(
    title: &'static str,
    description: &'static str,
    control: VegetationSliderControl,
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    precision: i32,
) -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            min_width: px(0),
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
        }
        Children [
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(2),
            }
            Children [
                @label(title)
                --
                @label_dim(description)
            ]
            --
            @FeathersSlider {
                @min: min,
                @max: max,
            }
            SliderValue(value)
            SliderStep(step)
            SliderPrecision(precision)
            control
            on(vegetation_slider_value_changed)
        ]
    }
}

fn vegetation_slider_value_changed(
    event: On<ValueChange<f32>>,
    controls: Query<&VegetationSliderControl, With<FeathersSlider>>,
    mut tuning: ResMut<VegetationTuning>,
    mut commands: Commands,
) {
    let Ok(control) = controls.get(event.source) else {
        return;
    };
    if !event.value.is_finite() {
        return;
    }

    match control {
        VegetationSliderControl::GlobalDensity => tuning.global_density = event.value.clamp(0.0, 1.4),
        VegetationSliderControl::ViewDistance => tuning.view_distance = event.value.clamp(18.0, 52.0),
        VegetationSliderControl::BladeHeight => tuning.blade_height = event.value.clamp(0.35, 1.6),
        VegetationSliderControl::GrasslandDensity => tuning.grassland_density = event.value.clamp(0.0, 1.8),
        VegetationSliderControl::WoodlandDensity => tuning.woodland_density = event.value.clamp(0.0, 1.8),
        VegetationSliderControl::DrylandDensity => tuning.dryland_density = event.value.clamp(0.0, 1.8),
        VegetationSliderControl::HighlandDensity => tuning.highland_density = event.value.clamp(0.0, 1.8),
        VegetationSliderControl::WindStrength => tuning.wind_strength = event.value.clamp(0.0, 1.0),
        VegetationSliderControl::WindSpeed => tuning.wind_speed = event.value.clamp(0.0, 12.0),
    }
    commands.entity(event.source).insert(SliderValue(event.value));
}

fn vegetation_shadow_toggle() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(10),
        }
        Children [
            @FeathersCheckbox
            VegetationToggle::Shadows
            on(vegetation_toggle_changed)
            --
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
                flex_grow: 1.0,
            }
            Children [
                @label("Grass shadows")
                --
                @label_dim("Enable near-field blade shadows; costs extra GPU time.")
            ]
        ]
    }
}

fn vegetation_toggle_changed(
    event: On<ValueChange<bool>>,
    controls: Query<&VegetationToggle, With<FeathersCheckbox>>,
    mut tuning: ResMut<VegetationTuning>,
) {
    let Ok(control) = controls.get(event.source) else {
        return;
    };
    match control {
        VegetationToggle::Shadows => tuning.shadows = event.value,
    }
}

/// Player-facing settings shown when interacting with an exploration portal.
fn exploration_setup_panel() -> impl Scene {
    let defaults = ExplorationPortal::default();
    let default_seed = defaults
        .seed
        .map_or(0, |seed| seed.min(i64::MAX as u64) as i64);

    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            right: px(0),
            bottom: px(0),
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(18)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor(Color::srgba(0.025, 0.035, 0.05, 0.82))
        bevy::ui::GlobalZIndex(22)
        ExplorationSetupRoot
        RelativeCursorPosition
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Node {
                width: px(680),
                max_width: percent(96),
                max_height: percent(94),
                min_width: px(0),
                min_height: px(0),
            }
            Children [
                @pane()
                Node {
                    display: Display::Flex,
                    width: percent(100),
                    max_height: percent(100),
                    min_width: px(0),
                    min_height: px(0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    overflow: Overflow::clip(),
                    border_radius: BorderRadius::all(px(8)),
                }
                ThemeBackgroundColor(tokens::PANE_BODY_BG)
                Propagate::<ThemeContext>(ThemeContext(SurfaceLevel::Floating))
                Children [
                    @pane_header()
                    Children [
                        @label("Explore the wilderness")
                        TextFont { font_size: px(18) }
                    ]
                    --
                    Node {
                        width: percent(100),
                        flex_grow: 1.0,
                        min_width: px(0),
                        min_height: px(0),
                        overflow: Overflow::scroll(),
                    }
                    Children [
                        @pane_body()
                        Children [
                            @label_dim("Adjust the map before entering. Seed 0 is always random; Randomize seed picks a value you can reuse.")
                            --
                            @exploration_integer_control(
                                "Seed",
                                "0 = random; a positive value reproduces the same map",
                                ExplorationParameterControl::Seed,
                                default_seed,
                                0,
                                i64::MAX,
                                1.0
                            )
                            --
                            @exploration_integer_control(
                                "Hex radius",
                                "Tiles from the center to each edge (8–32)",
                                ExplorationParameterControl::Radius,
                                defaults.radius as i64,
                                8,
                                32,
                                1.0
                            )
                            --
                            @exploration_float_control(
                                "Tile radius (m)",
                                "Rendered tile radius; centers use regular hex spacing",
                                ExplorationParameterControl::TileSize,
                                defaults.tile_size,
                                4.0,
                                8.0,
                                0.25,
                                2
                            )
                            --
                            @exploration_float_control(
                                "Height multiplier",
                                "Maximum terrain elevation scale",
                                ExplorationParameterControl::HeightMultiplier,
                                defaults.height_multiplier,
                                0.0,
                                5.0,
                                0.1,
                                2
                            )
                            --
                            @exploration_float_control(
                                "Maximum jump height (m)",
                                "Largest elevation difference the player can jump between adjacent tiles",
                                ExplorationParameterControl::MaxJumpHeight,
                                defaults.max_jump_height,
                                0.25,
                                3.5,
                                0.1,
                                2
                            )
                            --
                            @exploration_float_control(
                                "Terrain frequency",
                                "Scale of hills and elevation features",
                                ExplorationParameterControl::TerrainFrequency,
                                defaults.terrain_frequency,
                                0.01,
                                0.15,
                                0.001,
                                3
                            )
                            --
                            @exploration_float_control(
                                "Biome scale",
                                "Scale of woodland, grassland, and dryland regions",
                                ExplorationParameterControl::BiomeScale,
                                defaults.biome_scale,
                                0.006,
                                0.12,
                                0.001,
                                3
                            )

                            --
                            Text("Estimated map size")
                            template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().ink)))
                            TextFont { font_size: px(13) }
                            --
                            Text("469 hexes")
                            ExplorationSetupTileCountText
                            TextFont { font_size: px(11) }
                            template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
                        ]
                    ]
                    --
                    Node {
                        width: percent(100),
                        padding: UiRect::axes(px(12), px(10)),
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        justify_content: JustifyContent::FlexEnd,
                        align_items: AlignItems::Center,
                        column_gap: px(8),
                        row_gap: px(8),
                    }
                    Children [
                        @action_button("Randomize seed", CarbonIcon::Compass, CarbonButtonTone::Secondary)
                        on(|_activate: On<Activate>, mut draft: ResMut<ExplorationSetupViewModel>| {
                            draft.randomize_seed();
                        })
                        --
                        @action_button("Reset", CarbonIcon::Back, CarbonButtonTone::Ghost)
                        on(|_activate: On<Activate>, mut draft: ResMut<ExplorationSetupViewModel>| {
                            draft.reset();
                        })
                        --
                        @action_button("Cancel", CarbonIcon::Close, CarbonButtonTone::Secondary)
                        on(|_activate: On<Activate>, mut ui: ResMut<UiStack>| {
                            ui.remove(UiLayer::ExplorationSetup);
                        })
                        --
                        @action_button("Generate map", CarbonIcon::Confirm, CarbonButtonTone::Primary)
                        on(|_activate: On<Activate>,
                            draft: Res<ExplorationSetupViewModel>,
                            mut request: ResMut<ExplorationTravelRequest>,
                            mut ui: ResMut<UiStack>| {
                            request.0 = Some(PortalAction::Enter(draft.config.sanitized()));
                            ui.remove(UiLayer::ExplorationSetup);
                        })
                    ]
                ]
            ]
        ]
    }
}

fn exploration_integer_control(
    title: &'static str,
    description: &'static str,
    control: ExplorationParameterControl,
    value: i64,
    min: i64,
    max: i64,
    step: f64,
) -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            min_width: px(0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            column_gap: px(14),
            row_gap: px(6),
        }
        Children [
            Node {
                width: px(230),
                min_width: px(170),
                flex_direction: FlexDirection::Column,
                row_gap: px(3),
            }
            Children [
                @label(title)
                --
                @label_dim(description)
            ]
            --
            @FeathersNumberInput
            template(move |_| Ok(control))
            NumberInputValue::I64(value)
            HardLimit::i64(min..=max)
            NumberInputStep(step)
            NumberInputPrecision(0)
            Node {
                flex_grow: 1.0,
                min_width: px(140),
            }
            on(exploration_integer_value_changed)
        ]
    }
}

fn exploration_float_control(
    title: &'static str,
    description: &'static str,
    control: ExplorationParameterControl,
    value: f32,
    min: f32,
    max: f32,
    step: f64,
    precision: i32,
) -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            min_width: px(0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            column_gap: px(14),
            row_gap: px(6),
        }
        Children [
            Node {
                width: px(230),
                min_width: px(170),
                flex_direction: FlexDirection::Column,
                row_gap: px(3),
            }
            Children [
                @label(title)
                --
                @label_dim(description)
            ]
            --
            @FeathersNumberInput
            template(move |_| Ok(control))
            NumberInputValue::F32(value)
            HardLimit::f32(min..=max)
            NumberInputStep(step)
            NumberInputPrecision(precision)
            Node {
                flex_grow: 1.0,
                min_width: px(140),
            }
            on(exploration_float_value_changed)
        ]
    }
}

fn exploration_integer_value_changed(
    event: On<ValueChange<i64>>,
    controls: Query<&ExplorationParameterControl, With<FeathersNumberInput>>,
    mut draft: ResMut<ExplorationSetupViewModel>,
    mut commands: Commands,
) {
    let Ok(control) = controls.get(event.source) else {
        return;
    };

    match control {
        ExplorationParameterControl::Seed => {
            draft.config.seed = (event.value > 0).then_some(event.value as u64);
        }
        ExplorationParameterControl::Radius => {
            draft.config.radius = event.value.clamp(8, 32) as u32;
        }
        _ => return,
    }

    commands
        .entity(event.source)
        .insert(NumberInputValue::I64(event.value));
}

fn exploration_float_value_changed(
    event: On<ValueChange<f32>>,
    controls: Query<&ExplorationParameterControl, With<FeathersNumberInput>>,
    mut draft: ResMut<ExplorationSetupViewModel>,
    mut commands: Commands,
) {
    if !event.value.is_finite() {
        return;
    }
    let Ok(control) = controls.get(event.source) else {
        return;
    };

    match control {
        ExplorationParameterControl::TileSize => {
            draft.config.tile_size = event.value.clamp(4.0, 8.0);
        }
        ExplorationParameterControl::HeightMultiplier => {
            draft.config.height_multiplier = event.value.clamp(0.0, 5.0);
        }
        ExplorationParameterControl::TerrainFrequency => {
            draft.config.terrain_frequency = event.value.clamp(0.01, 0.15);
        }
        ExplorationParameterControl::BiomeScale => {
            draft.config.biome_scale = event.value.clamp(0.006, 0.12);
        }
        ExplorationParameterControl::MaxJumpHeight => {
            draft.config.max_jump_height = event.value.clamp(0.25, 3.5);
        }
        _ => return,
    }

    commands
        .entity(event.source)
        .insert(NumberInputValue::F32(event.value));
}

fn minimap() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            bottom: px(16),
            right: px(16),
            width: px(144),
            height: px(144),
            padding: UiRect::all(px(8)),
            border_radius: BorderRadius::all(px(72)),
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().panel)))
        bevy::ui::GlobalZIndex(2)
        RelativeCursorPosition
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Node {
                position_type: PositionType::Relative,
                width: px(120),
                height: px(120),
                overflow: Overflow::clip(),
                border_radius: BorderRadius::all(px(60)),
            }
            template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().surface_deep)))
            Children [
                @minimap_exploration_tiles()
                --
                Node {
                    position_type: PositionType::Absolute,
                    left: px(56),
                    top: px(56),
                    width: px(8),
                    height: px(8),
                    border_radius: BorderRadius::all(px(4)),
                }
                template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().ink)))
                --
                @minimap_zone_dot(ZoneKind::Shop)
                --
                @minimap_zone_dot(ZoneKind::Guild)
            ]
        ]
    }
}

fn minimap_exploration_tiles() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px(120),
            height: px(120),
        }
        Children [
                @minimap_exploration_dot(-3, 0)
                --
                @minimap_exploration_dot(-3, 1)
                --
                @minimap_exploration_dot(-3, 2)
                --
                @minimap_exploration_dot(-3, 3)
                --
                @minimap_exploration_dot(-2, -1)
                --
                @minimap_exploration_dot(-2, 0)
                --
                @minimap_exploration_dot(-2, 1)
                --
                @minimap_exploration_dot(-2, 2)
                --
                @minimap_exploration_dot(-2, 3)
                --
                @minimap_exploration_dot(-1, -2)
                --
                @minimap_exploration_dot(-1, -1)
                --
                @minimap_exploration_dot(-1, 0)
                --
                @minimap_exploration_dot(-1, 1)
                --
                @minimap_exploration_dot(-1, 2)
                --
                @minimap_exploration_dot(-1, 3)
                --
                @minimap_exploration_dot(0, -3)
                --
                @minimap_exploration_dot(0, -2)
                --
                @minimap_exploration_dot(0, -1)
                --
                @minimap_exploration_dot(0, 0)
                --
                @minimap_exploration_dot(0, 1)
                --
                @minimap_exploration_dot(0, 2)
                --
                @minimap_exploration_dot(0, 3)
                --
                @minimap_exploration_dot(1, -3)
                --
                @minimap_exploration_dot(1, -2)
                --
                @minimap_exploration_dot(1, -1)
                --
                @minimap_exploration_dot(1, 0)
                --
                @minimap_exploration_dot(1, 1)
                --
                @minimap_exploration_dot(1, 2)
                --
                @minimap_exploration_dot(2, -3)
                --
                @minimap_exploration_dot(2, -2)
                --
                @minimap_exploration_dot(2, -1)
                --
                @minimap_exploration_dot(2, 0)
                --
                @minimap_exploration_dot(2, 1)
                --
                @minimap_exploration_dot(3, -3)
                --
                @minimap_exploration_dot(3, -2)
                --
                @minimap_exploration_dot(3, -1)
                --
                @minimap_exploration_dot(3, 0)
        ]
    }
}

fn minimap_exploration_dot(q_offset: i32, r_offset: i32) -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px(6),
            height: px(6),
            border_radius: BorderRadius::all(px(3)),
        }
        MinimapExplorationDot(q_offset, r_offset)
        BackgroundColor(Color::WHITE)
        Visibility::Hidden
    }
}

fn minimap_zone_dot(kind: ZoneKind) -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px(7),
            height: px(7),
            border_radius: BorderRadius::all(px(4)),
        }
        MinimapZoneDot(kind)
        Visibility::Hidden
        template(move |ctx| {
            let theme = ctx.resource::<CarbonTheme>();
            let color = match kind {
                ZoneKind::Shop => theme.accent,
                ZoneKind::Guild => theme.warning,
            };
            Ok(BackgroundColor(color))
        })
    }
}

fn inventory_scene() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(20)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().overlay)))
        bevy::ui::GlobalZIndex(12)
        InventoryRoot
        RelativeCursorPosition
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Node {
                width: percent(100),
                height: percent(100),
                max_width: px(1160),
                max_height: px(780),
                min_width: px(0),
                min_height: px(0),
                overflow: Overflow::clip(),
                padding: UiRect::all(px(14)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: px(14),
                border_radius: BorderRadius::all(px(9)),
            }
            template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().panel)))
            Children [
                @page_header(
                    "Inventory",
                    "Party inventory",
                    "Review carried items, equipment, and party resources. Item tiles are visual inventory entries."
                )
                --
                Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: px(12),
                    row_gap: px(8),
                }
                Children [
                    @key_hint("RMB", "Close inventory")
                    --
                    Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        align_items: AlignItems::Center,
                        column_gap: px(8),
                    }
                    Children [
                        @action_button("Close", CarbonIcon::Close, CarbonButtonTone::Secondary)
                        on(|_activate: On<Activate>, mut ui: ResMut<UiStack>| {
                            ui.remove(UiLayer::Inventory);
                        })
                    ]
                ]
                --
                Node {
                    width: percent(100),
                    flex_grow: 1.0,
                    min_width: px(0),
                    min_height: px(0),
                    overflow: Overflow::scroll(),
                    flex_direction: FlexDirection::Column,
                }
                Children [
                    Node {
                        width: percent(100),
                        min_width: px(0),
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        align_content: AlignContent::Start,
                        align_items: AlignItems::Stretch,
                        column_gap: px(14),
                        row_gap: px(14),
                    }
                    Children [
                        Node {
                            width: px(250),
                            flex_basis: px(240),
                            min_width: px(190),
                            flex_shrink: 1.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: px(12),
                        }
                        Children [
                            @character_panel()
                            --
                            @resources_panel()
                        ]
                        --
                        Node {
                            flex_grow: 1.0,
                            flex_basis: px(420),
                            min_width: px(190),
                            flex_shrink: 1.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: px(12),
                        }
                        Children [
                            @inventory_items()
                        ]
                    ]
                ]
            ]
        ]
    }
}

fn character_panel() -> impl Scene {
    bsn! {
        @panel(percent(100))
        Children [
            @section_header(CarbonIcon::Character, "Character")
            --
            Node {
                width: percent(100),
                height: px(190),
                min_width: px(0),
                border_radius: BorderRadius::all(px(6)),
                overflow: Overflow::clip(),
            }
            template(|ctx| {
                Ok(ImageNode::new(
                    ctx.resource::<AssetServer>().load("character/portrait.png"),
                ))
            })
            --
            @section_header(CarbonIcon::Shield, "Equipment")
            --
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_content: AlignContent::Start,
                column_gap: px(7),
                row_gap: px(7),
            }
            Children [
                @gear_slot("Neck", CarbonIcon::Compass)
                --
                @gear_slot("Head", CarbonIcon::Character)
                --
                @gear_slot("Shoulders", CarbonIcon::Shield)
                --
                @gear_slot("L Arm", CarbonIcon::Shield)
                --
                @gear_slot("Torso", CarbonIcon::Character)
                --
                @gear_slot("R Arm", CarbonIcon::Shield)
                --
                @gear_slot("Legs", CarbonIcon::Compass)
                --
                @gear_slot("Boots", CarbonIcon::Stamina)
            ]
        ]
    }
}

fn resources_panel() -> impl Scene {
    bsn! {
        @panel(percent(100))
        Children [
            @section_header(CarbonIcon::Coin, "Resources")
            --
            Text("Gold: 0")
            CurrencyLabel::Gold
            TextFont { font_size: px(12) }
            template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
            --
            Text("Wood: 0")
            CurrencyLabel::Wood
            TextFont { font_size: px(12) }
            template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
            --
            Text("Stone: 0")
            CurrencyLabel::Stone
            TextFont { font_size: px(12) }
            template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
        ]
    }
}

fn inventory_items() -> impl Scene {
    bsn! {
        @panel(percent(100))
        Children [
            @section_header(CarbonIcon::Inventory, "Carried items")
            --
            Node {
                width: percent(100),
                min_width: px(0),
                min_height: px(112),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_content: AlignContent::Start,
                align_items: AlignItems::Start,
                column_gap: px(8),
                row_gap: px(8),
            }
            Children [
                @item_art_tile("items/evil-pup.png", "Evil pup", None)
                --
                @item_art_tile("items/maurice.webp", "Maurice", None)
                --
                @item_art_tile("items/salad.jpg", "Salad", None)
                --
                @item_art_tile("items/scooby.webp", "Scooby", None)
                --
                @item_art_tile("items/theo.jpg", "Theo", None)
            ]
        ]
    }
}

fn player_settings_panel() -> impl Scene {
    let defaults = crate::gameplay::player::PlayerSettings::default();
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(0), left: px(0), width: percent(100), height: percent(100),
            padding: UiRect::all(px(18)), justify_content: JustifyContent::Center, align_items: AlignItems::Center,
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().overlay)))
        bevy::ui::GlobalZIndex(32)
        SettingsPanelRoot
        RelativeCursorPosition
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Node { width: px(620), max_width: percent(96), max_height: percent(94), min_width: px(0), overflow: Overflow::scroll() }
            Children [
                @panel(percent(100))
                Children [
                    @page_title("Settings")
                    --
                    @action_button("Restore defaults", CarbonIcon::Back, CarbonButtonTone::Ghost)
                    on(|_activate: On<Activate>, mut settings: ResMut<crate::gameplay::player::PlayerSettings>| { *settings = default(); })
                    --
                    @section_header(CarbonIcon::Settings, "Camera")
                    --
                    @player_setting_control("Pan sensitivity", "Middle-mouse drag speed.", PlayerSettingControl::PanSensitivity, defaults.camera_pan_sensitivity, 0.0002, 0.01, 0.0001, 4)
                    --
                    @player_setting_control("Zoom sensitivity", "Scroll-wheel zoom per line.", PlayerSettingControl::ZoomSensitivity, defaults.camera_zoom_sensitivity, 0.1, 2.0, 0.05, 2)
                    --
                    @player_setting_control("Tilt scroll speed", "Camera tilt change per scroll line.", PlayerSettingControl::PitchSensitivity, defaults.camera_pitch_sensitivity, 0.002, 0.06, 0.002, 3)
                    --
                    @player_auto_lock_setting()
                    --
                    @section_header(CarbonIcon::Character, "Movement")
                    --
                    @player_setting_control("Walk speed", "Normal keyboard and click-to-move speed.", PlayerSettingControl::WalkSpeed, defaults.walk_speed, 1.0, 12.0, 0.25, 2)
                    --
                    @player_setting_control("Sprint speed", "Speed while holding Shift.", PlayerSettingControl::SprintSpeed, defaults.sprint_speed, 1.0, 20.0, 0.25, 2)
                    --
                    @action_button("Developer settings", CarbonIcon::Settings, CarbonButtonTone::Secondary)
                    on(|_activate: On<Activate>, mut state: ResMut<DeveloperSettingsPanelState>| { state.expanded = !state.expanded; })
                    --
                    Node { width: percent(100), padding: UiRect::all(px(10)), flex_direction: FlexDirection::Column, row_gap: px(8) }
                    DeveloperSettingsContentRoot
                    Visibility::Hidden
                    Children [
                        @eyebrow_text("DEVELOPER")
                        --
                        @muted_text("Experimental controls for debugging the current session.", 12.0)
                        --
                        @action_button("Toggle exploration debug panel", CarbonIcon::Map, CarbonButtonTone::Secondary)
                        on(|_activate: On<Activate>, mut state: ResMut<ExplorationDebugPanelState>| { state.visible = !state.visible; })
                        --
                        @action_button("Reset free-camera offset", CarbonIcon::Back, CarbonButtonTone::Secondary)
                        on(|_activate: On<Activate>, mut cameras: Query<&mut crate::gameplay::player::GameplayCamera>| { if let Ok(mut camera) = cameras.single_mut() { camera.pan_offset = Vec3::ZERO; } })
                    ]
                    --
                    @action_button("Back", CarbonIcon::Back, CarbonButtonTone::Primary)
                    on(|_activate: On<Activate>, mut ui: ResMut<UiStack>| { ui.remove(UiLayer::Settings); })
                ]
            ]
        ]
    }
}

pub(in crate::ui) fn player_setting_control(
    title: &'static str,
    description: &'static str,
    control: PlayerSettingControl,
    value: f32,
    min: f32,
    max: f32,
    step: f64,
    precision: i32,
) -> impl Scene {
    bsn! {
        Node { width: percent(100), min_width: px(0), flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, align_items: AlignItems::Center, column_gap: px(14), row_gap: px(6) }
        Children [
            Node { width: px(250), min_width: px(170), flex_direction: FlexDirection::Column, row_gap: px(3) }
            Children [ @label(title) -- @label_dim(description) ]
            --
            @FeathersNumberInput
            template(move |_| Ok(control))
            NumberInputValue::F32(value)
            HardLimit::f32(min..=max)
            NumberInputStep(step)
            NumberInputPrecision(precision)
            Node { flex_grow: 1.0, min_width: px(140) }
            on(player_setting_value_changed)
        ]
    }
}

pub(in crate::ui) fn player_auto_lock_setting() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            min_width: px(0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(14),
        }
        Children [
            @FeathersCheckbox
            PlayerSettingToggle::CameraAutoLock
            Checked
            on(player_setting_toggle_changed)
            --
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(3),
                flex_grow: 1.0,
            }
            Children [
                @label("Auto-lock camera while moving")
                --
                @label_dim("Recenters the camera when keyboard or click movement begins.")
            ]
        ]
    }
}

pub(in crate::ui) fn player_setting_toggle_changed(
    event: On<ValueChange<bool>>,
    controls: Query<&PlayerSettingToggle, With<FeathersCheckbox>>,
    mut settings: ResMut<crate::gameplay::player::PlayerSettings>,
) {
    let Ok(control) = controls.get(event.source) else {
        return;
    };
    match control {
        PlayerSettingToggle::CameraAutoLock => settings.camera_auto_lock = event.value,
    }
}

pub(in crate::ui) fn player_setting_value_changed(
    event: On<ValueChange<f32>>,
    controls: Query<&PlayerSettingControl, With<FeathersNumberInput>>,
    mut settings: ResMut<crate::gameplay::player::PlayerSettings>,
) {
    if !event.value.is_finite() {
        return;
    }
    let Ok(control) = controls.get(event.source) else {
        return;
    };
    match control {
        PlayerSettingControl::PanSensitivity => {
            settings.camera_pan_sensitivity = event.value.clamp(0.0002, 0.01)
        }
        PlayerSettingControl::ZoomSensitivity => {
            settings.camera_zoom_sensitivity = event.value.clamp(0.1, 2.0)
        }
        PlayerSettingControl::PitchSensitivity => {
            settings.camera_pitch_sensitivity = event.value.clamp(0.002, 0.06)
        }
        PlayerSettingControl::WalkSpeed => settings.walk_speed = event.value.clamp(1.0, 12.0),
        PlayerSettingControl::SprintSpeed => settings.sprint_speed = event.value.clamp(1.0, 20.0),
    }
}
fn game_menu_modal() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(20)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().overlay)))
        bevy::ui::GlobalZIndex(30)
        GameMenuModal
        RelativeCursorPosition
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Node {
                width: px(360),
                max_width: percent(92),
                min_width: px(0),
                max_height: percent(90),
                overflow: Overflow::scroll(),
            }
            Children [
                @panel(percent(100))
                Children [
                    @page_title("Game menu")
                    --
                    @muted_text("The current session is suspended while this menu is open.", 12.0)
                    --
                    @game_menu_action("Continue", CarbonIcon::Back, GameMenuAction::Continue, CarbonButtonTone::Primary)
                    --
                    @action_button("Settings", CarbonIcon::Settings, CarbonButtonTone::Secondary)
                    on(|_activate: On<Activate>, mut ui: ResMut<UiStack>| { ui.push(UiLayer::Settings); })
                    --
                    @game_menu_action("Exit to main menu", CarbonIcon::Compass, GameMenuAction::ExitToMain, CarbonButtonTone::Secondary)
                    --
                    @game_menu_action("Exit to desktop", CarbonIcon::Close, GameMenuAction::ExitToDesktop, CarbonButtonTone::Danger)
                ]
            ]
        ]
    }
}

#[derive(Clone, Copy)]
enum GameMenuAction {
    Continue,
    ExitToMain,
    ExitToDesktop,
}

fn game_menu_action(
    label: &'static str,
    icon: CarbonIcon,
    action: GameMenuAction,
    tone: CarbonButtonTone,
) -> impl Scene {
    bsn! {
        @action_button(label, icon, tone)
        on(move |_activate: On<Activate>,
            mut ui: ResMut<UiStack>,
            mut next: ResMut<NextState<AppState>>,
            mut exit: MessageWriter<AppExit>| {
            ui.remove(UiLayer::GameMenu);
            match action {
                GameMenuAction::Continue => {}
                GameMenuAction::ExitToMain => {
                    ui.reset();
                    next.set(AppState::MainMenu);
                }
                GameMenuAction::ExitToDesktop => {
                    exit.write(AppExit::Success);
                }
            }
        })
    }
}

fn pause_overlay() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(20)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().overlay)))
        bevy::ui::GlobalZIndex(25)
        PauseOverlay
        RelativeCursorPosition
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Node {
                width: px(340),
                max_width: percent(92),
                min_width: px(0),
                max_height: percent(90),
                overflow: Overflow::scroll(),
            }
            Children [
                @panel(percent(100))
                Children [
                    @page_title("Paused")
                    --
                    @muted_text("Press P to resume or use the actions below.", 12.0)
                    --
                    @action_button("Resume", CarbonIcon::Back, CarbonButtonTone::Primary)
                    on(|_activate: On<Activate>, mut ui: ResMut<UiStack>| {
                        ui.remove(UiLayer::Pause);
                    })
                    --
                    @action_button("Open game menu", CarbonIcon::Settings, CarbonButtonTone::Secondary)
                    on(|_activate: On<Activate>, mut ui: ResMut<UiStack>| {
                        ui.push(UiLayer::GameMenu);
                    })
                ]
            ]
        ]
    }
}
