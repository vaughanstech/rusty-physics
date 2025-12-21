use avian3d::prelude::*;
use bevy::{ color, diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}, prelude::* };
use bevy_asset::{AssetServer};
use bevy_egui::{EguiPrimaryContextPass, EguiPlugin};
use bevy_framepace::*;

use crate::{game::pause_menu::InGameMenuState, interaction_modes::*, interactive_menu::*, peripherals::*};
use super::SetMaxFps;

use super::GameState;

#[derive(Component)]
pub struct FpsText;

pub fn game_plugin(
    app: &mut App,
) {
    app
        .add_systems(Startup, game_setup)
        .insert_resource(CameraSettings {
            speed: 8.0,
            sensitivity: 0.002,
            zoom_speed: 30.0,
        })
        .add_plugins(EguiPlugin::default())
        .insert_resource(ImpulseSettings::default())
        .insert_resource(CameraOrientation::default())
        .insert_resource(CursorDistance(10.0)) // set cursor distance on spawn
        .insert_resource(InteractionMode(InteractionModeType::Click))
        .add_systems(EguiPrimaryContextPass, interactive_menu.run_if(in_state(GameState::Game)))
        .add_systems(Update, (
            // spawn_cubes.run_if(on_timer(Duration::from_secs(1))),
            keyboard_movement,
            mouse_look,
            // Camera Zoom/Scroll runs only in Click Mode
            (
                mouse_scroll,
                set_impulse_cursor_visibility::<false>,
                set_wrecker_cursor_visibility::<false>,
            ).run_if(resource_equals(InteractionMode(InteractionModeType::Click))),
            // Cursor Control/Draw runs only in Impulse && Wrecker Mode
            (
                scroll_control, // System to update cursor distance
                draw_impulse_cursor,                // System to draw the gizmo
                apply_force,
                set_impulse_cursor_visibility::<true>,
                set_wrecker_cursor_visibility::<false>,
            ).run_if(resource_equals(InteractionMode(InteractionModeType::Impulse))),
            (
                scroll_control,
                draw_wrecker_cursor,
                set_impulse_cursor_visibility::<false>,
                set_wrecker_cursor_visibility::<true>,
            ).run_if(resource_equals(InteractionMode(InteractionModeType::Wrecker))),
            toggle_debug_render_state,
            game_action,
        ).run_if(in_state(GameState::Game)));
}

fn game_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    // Light: bright white light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    commands.spawn((
        Text::new("FPS: "),
        TextFont {
            font_size: 42.0,
            ..default()
        },
    ))
    .with_child((
        TextSpan::default(),
        TextFont {
            font_size: 33.0,
            ..Default::default()
        },
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        FpsText,
    ));

    let impulse_ball = commands.spawn((
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(5).from_asset("shapes.glb"))
        ),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::Hidden,
    ))
    .insert(ImpulseCursor).id();

    let sphere = meshes.add(Sphere::new(0.5));

    let wrecker_ball = commands.spawn((
        Mesh3d(sphere.clone()),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
        Transform::from_xyz(0.0, 10.0, 0.0),
        Collider::sphere(0.5),
        RigidBody::Kinematic,
        Visibility::Hidden,
    ))
    .insert(WreckerCursor).id();

    let text_style = TextFont {
        font: asset_server.load(r"fonts\FiraMono-Medium.ttf"),
        ..Default::default()
    };

    let label_text_style = (text_style.clone(), TextColor(color::palettes::css::ORANGE.into()));


    let mut impulse_label = |entity: Entity, label: &str| {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            ExampleLabel { entity },
            children![(
                Text::new(label),
                label_text_style.clone(),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::ZERO,
                    ..Default::default()
                },
                TextLayout::default().with_no_wrap(),
                ImpulseCoords,
            )],
            Visibility::Hidden,
        )).insert(ImpulseCursor);
    };
    impulse_label(impulse_ball, "┌─ Impulse: (0.00, 0.00, 0.00)");

    let mut wrecker_label = |entity: Entity, label: &str| {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            ExampleLabel { entity },
            children![(
                Text::new(label),
                label_text_style.clone(),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::ZERO,
                    ..Default::default()
                },
                TextLayout::default().with_no_wrap(),
                WreckerCoords,
            )],
            Visibility::Hidden,
        )).insert(WreckerCursor);
    };
    wrecker_label(wrecker_ball, "┌─ Wrecker: (0.00, 0.00, 0.00)");
}

/// Set the max framerate limit
pub fn set_max_fps(
    mut commands: Commands,
    mut settings: ResMut<FramepaceSettings>,
    fps_limit: Res<SetMaxFps>,
) {
    // Setting the Physics time equal to the max framerate
    commands.insert_resource(Time::<Fixed>::from_hz(fps_limit.fps));
    // Actually setting global max fps
    settings.limiter = Limiter::from_framerate(fps_limit.fps);
}

/// Tracks frames per second
pub fn fps_counter(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut TextSpan, With<FpsText>>,
) {
    for mut span in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS)
            && let Some(value) = fps.smoothed() 
        {
            // update the value of the second section
            **span = format!("{value:.2}");
        }
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum InGameState {
    Playing,
    #[default]
    Disabled,
}

/// Cross-system function used to toggle between the Game state and the Pause state
pub fn game_action(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut menu_state: ResMut<NextState<InGameMenuState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match state.get() {
            GameState::Game => {
                game_state.set(GameState::Paused);
                
                info!("Pausing Game");
            }
            GameState::Paused => {
                game_state.set(GameState::Game);
                menu_state.set(InGameMenuState::Disabled);
                info!("Resuming Game");
            }
            _ => {}
        }
        // for mut menu_toggle in menu_toggle_query.iter_mut() {
        //     *menu_toggle = match *menu_toggle {
        //         TogglePauseMenu::Disabled => {
        //             game_state.set(GameState::Paused);
        //             info!("Game state = Paused");
        //             TogglePauseMenu::Enabled
        //         }
        //         TogglePauseMenu::Enabled => {
        //             game_state.set(GameState::Game);
        //             menu_state.set(InGameMenuState::Disabled);
        //             info!("Game state = Game");
        //             TogglePauseMenu::Disabled
        //         }
        //     }
        // }
    }
}

fn toggle_debug_render_state(
    // mut debug_render_state: ResMut<DebugRenderState>,
    mut gizmo_config_store: ResMut<GizmoConfigStore>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        let (config, _) = gizmo_config_store
            .config_mut::<PhysicsGizmos>();
        config.enabled = !config.enabled;
    }
}

pub mod pause_menu {
    use bevy::{color, prelude::*};

    use super::GameState;

    use crate::{menu::MenuState, game::game_action};

    /// This plugin manages the in-game Pause menu
    pub fn pause_menu_plugin(
        app: &mut App,
    ) {
        app
            .init_state::<InGameMenuState>()
            .add_systems(OnEnter(GameState::Paused), menu_setup)
            .add_systems(OnEnter(InGameMenuState::Base), in_game_menu_setup)
            .add_systems(Update, (menu_action, button_system, game_action).run_if(in_state(GameState::Paused)));
    }

    /// State used for the current pause menu screen
    #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
    pub enum InGameMenuState {
        Base,
        Settings,
        #[default]
        Disabled,
    }

    /// Tag component used to tag entities added on the in-game menu screen
    #[derive(Component)]
    struct OnInGameMenuScreen;

    /// Tag component used to tag entities added on the in-game settings menu screen
    #[derive(Component)]
    struct OnSettingsMenuScreen;

    const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
    const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
    const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
    const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

    /// Tag component used to mark which setting is currently selected
    #[derive(Component)]
    struct SelectedOption;

    /// All actions that can be triggered from a button click
    #[derive(Component)]
    enum InGameMenuButtonAction {
        Resume,
        Settings,
        BackToInGameMenu,
        BackToMainMenu,
        Quit,
    }

    /// This system handles changing all buttons color based on mouse interaction
    fn button_system(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        for (interaction, mut background_color, selected) in &mut interaction_query {
            *background_color = match (*interaction, selected) {
                (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
                (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
                (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
                (Interaction::None, None) => NORMAL_BUTTON.into(),
            }
        }
    }

    fn menu_setup(
        mut menu_state: ResMut<NextState<InGameMenuState>>,
    ) {
        menu_state.set(InGameMenuState::Base);
    }

    fn in_game_menu_setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        // Common style for all buttons on the screen
        let button_node = Node {
            width: px(300),
            height: px(65),
            margin: UiRect::all(px(20)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_icon_node = Node {
            width: px(30),
            // This takes the idons our of the flexbox flow, to be positioned exactly
            position_type: PositionType::Absolute,
            // The icon will be close to the left border of the button
            left: px(10),
            ..default()
        };
        let button_text_font = TextFont {
            font_size: 20.0,
            ..default()
        };

        let right_icon = asset_server.load(r"icons\chevron_right_icon.png");
        let settings_icon = asset_server.load(r"icons\settings_icon.png");
        let exit_icon = asset_server.load(r"icons\logout_icon.png");

        commands.spawn((
            DespawnOnExit(InGameMenuState::Base),
            Node {
                width: Val::Auto,
                height: vh(100),
                margin: UiRect {left: Val::Auto, ..Default::default()},
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(color::palettes::css::CRIMSON.into()),
            OnInGameMenuScreen,
            children![(
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                // BackgroundColor(color::palettes::css::CRIMSON.into()),
                children![
                    // Display the game name
                    (
                        Text::new("Rusty Physics"),
                        TextFont {
                            font_size: 50.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        Node {
                            margin: UiRect::all(px(100)),
                            ..default()
                        },
                    ),
                    // Display all buttons for each action available from the main menu
                    (
                        Button,
                        button_node.clone(),
                        BackgroundColor(NORMAL_BUTTON),
                        InGameMenuButtonAction::Resume,
                        children![
                            (ImageNode::new(right_icon), button_icon_node.clone()),
                            (
                                Text::new("New Game"),
                                button_text_font.clone(),
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ),
                        ]
                    ),
                    (
                        Button,
                        button_node.clone(),
                        BackgroundColor(NORMAL_BUTTON),
                        InGameMenuButtonAction::Settings,
                        children![
                            (ImageNode::new(settings_icon), button_icon_node.clone()),
                            (
                                Text::new("Settings"),
                                button_text_font.clone(),
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ),
                        ]
                    ),
                    (
                        Button,
                        button_node,
                        BackgroundColor(NORMAL_BUTTON),
                        InGameMenuButtonAction::Quit,
                        children![
                            (ImageNode::new(exit_icon), button_icon_node),
                            (Text::new("Quit"), button_text_font, TextColor(Color::srgb(0.9, 0.9, 0.9))),
                        ]
                    ),
                ]
            )],
        ));
    }

    fn menu_action(
        interaction_query: Query<
            (&Interaction, &InGameMenuButtonAction),
            (Changed<Interaction>, With<Button>),
        >,
        mut app_exit_writer: MessageWriter<AppExit>,
        mut paused_menu_state: ResMut<NextState<InGameMenuState>>,
        mut game_state: ResMut<NextState<GameState>>,
        mut menu_state: ResMut<NextState<MenuState>>,
    ) {
        for (interaction, in_game_menu_button_action) in &interaction_query {
            if *interaction == Interaction::Pressed {
                match in_game_menu_button_action {
                    InGameMenuButtonAction::Quit => {
                        app_exit_writer.write(AppExit::Success);
                    }
                    InGameMenuButtonAction::Resume => {
                        game_state.set(GameState::Game);
                        paused_menu_state.set(InGameMenuState::Disabled);
                    }
                    InGameMenuButtonAction::Settings => paused_menu_state.set(InGameMenuState::Settings),
                    InGameMenuButtonAction::BackToInGameMenu => paused_menu_state.set(InGameMenuState::Base),
                    InGameMenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                }
            }
        }
    }
}
