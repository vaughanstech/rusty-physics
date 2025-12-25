pub mod main_menu {
    use bevy::{color, prelude::*};

    use crate::SetFps;

    use crate::GameState;

    // This plugin manages the menu
    pub fn menu_plugin(
        app: &mut App,
    ) {
        app
            // At start, the menu is not enabled. This will be changed in `menu_setup` when entering the `GameState::Menu` state
            // Current screen in the menu is handled by an independent state from `GameState`
            .init_state::<MenuState>()
            .add_systems(OnEnter(GameState::Menu), menu_setup)
            // Systems to handle the main menu screen
            .add_systems(OnEnter(MenuState::Main), main_menu_setup)
            // Systems to handle the settings menu screen
            // .add_systems(OnEnter(MenuState::Settings), settings_menu_setup)
            // Systems to handle the display settings screen
            // .add_systems(
            //     OnEnter(MenuState::SettingsDisplay), 
            //     display_settings_menu_setup,
            // )
            // Systems to handle the sound settings screen
            // .add_systems(OnEnter(MenuState::SettingsSound), sound_settings_menu_setup)
            // Common systems to all screens that handles buttons behavior
            .add_systems(
                Update,
                (menu_action, button_system).run_if(in_state(GameState::Menu)),
            )
            .add_systems(OnEnter(MenuState::Settings), in_game_settings_menu_setup)
            .add_systems(Update, setting_button::<SetFps>.run_if(in_state(MenuState::Settings)));
    }

    /// State used for the current menu screen
    #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
    pub enum MenuState {
        Main,
        Settings,
        #[default]
        Disabled,
    }

    /// Tag component used to tag entities added on the main menu screen
    #[derive(Component)]
    struct OnMainMenuScreen;

    /// Tag component used to tag entities added on the settings menu screen
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
    enum MenuButtonAction {
        Play,
        Settings,
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

    /// This system updates the settings when a new value for a setting is selected
    /// and marks the button as the one currently selected
    fn setting_button<T: Resource + Component + PartialEq + Copy>(
        interaction_query: Query<(&Interaction, &T, Entity), (Changed<Interaction>, With<Button>)>,
        selected_query: Single<(Entity, &mut BackgroundColor), With<SelectedOption>>,
        mut commands: Commands,
        mut setting: ResMut<T>,
    ) {
        let (previous_button, mut previous_button_color) = selected_query.into_inner();
        for (interaction, button_setting, entity) in &interaction_query {
            if *interaction == Interaction::Pressed && *setting != *button_setting {
                *previous_button_color = NORMAL_BUTTON.into();
                commands.entity(previous_button).remove::<SelectedOption>();
                commands.entity(entity).insert(SelectedOption);
                *setting = *button_setting;
            }
        }
    }

    fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
        menu_state.set(MenuState::Main);
    }

    fn main_menu_setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>
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
            font_size: 33.0,
            ..default()
        };

        let right_icon = asset_server.load(r"icons\chevron_right_icon.png");
        let settings_icon = asset_server.load(r"icons\settings_icon.png");
        let exit_icon = asset_server.load(r"icons\logout_icon.png");

        commands.spawn((
            DespawnOnExit(MenuState::Main),
            Node {
                width: vw(100),
                height: vh(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(color::palettes::css::CRIMSON.into()),
            OnMainMenuScreen,
            children![(
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    // Display the game name
                    (
                        Text::new("Rusty Physics"),
                        TextFont {
                            font_size: 67.0,
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
                        MenuButtonAction::Play,
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
                        MenuButtonAction::Settings,
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
                        MenuButtonAction::Quit,
                        children![
                            (ImageNode::new(exit_icon), button_icon_node),
                            (Text::new("Quit"), button_text_font, TextColor(Color::srgb(0.9, 0.9, 0.9))),
                        ]
                    ),
                ]
            )],
        ));
    }

    fn in_game_settings_menu_setup(
        mut commands: Commands,
        fps_limit: Res<SetFps>,
    ) {
        fn button_node() -> Node {
            Node {
                width: px(200),
                height: px(65),
                margin: UiRect::all(px(20)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            }
        }
        fn button_text_style() -> impl Bundle {
            (
                TextFont {
                    font_size: 33.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            )
        }

        let fps_limit = *fps_limit;
        commands.spawn((
            DespawnOnExit(MenuState::Settings),
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(color::palettes::css::CRIMSON.into()),
            OnSettingsMenuScreen,
            children![(
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    // Creating a new `Node`, this time not setting its `flex_direction`
                    // Use the default value, `FlexDirection::Row`, from left to right.
                    (
                        Node {
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(color::palettes::css::CRIMSON.into()),
                        Children::spawn((
                            // Display a label for the current setting
                            Spawn((Text::new("FPS Limit"), button_text_style())),
                            SpawnWith(move |parent: &mut ChildSpawner| {
                                for fps_setting in [
                                    SetFps::Low,
                                    SetFps::Medium,
                                    SetFps::High,
                                    SetFps::Uncapped,
                                ] {
                                    let mut entity = parent.spawn((
                                        Button,
                                        Node {
                                            width: px(150),
                                            height: px(65),
                                            ..button_node()
                                        },
                                        BackgroundColor(NORMAL_BUTTON),
                                        fps_setting,
                                        children![(
                                            Text::new(format!("{fps_setting:?}")),
                                            button_text_style(),
                                        )],
                                    ));
                                    if fps_limit == fps_setting {
                                        entity.insert(SelectedOption);
                                    }
                                }
                            })
                        ))
                    ),
                    (
                        Button,
                        button_node(),
                        BackgroundColor(NORMAL_BUTTON),
                        MenuButtonAction::BackToMainMenu,
                        children![(Text::new("Back"), button_text_style())]
                    )
                ]
            )]
        ));
    }

    fn menu_action(
        interaction_query: Query<
            (&Interaction, &MenuButtonAction),
            (Changed<Interaction>, With<Button>),
        >,
        mut app_exit_writer: MessageWriter<AppExit>,
        mut menu_state: ResMut<NextState<MenuState>>,
        mut game_state: ResMut<NextState<GameState>>,
    ) {
        for (interaction, menu_button_action) in &interaction_query {
            if *interaction == Interaction::Pressed {
                match menu_button_action {
                    MenuButtonAction::Quit => {
                        app_exit_writer.write(AppExit::Success);
                    }
                    MenuButtonAction::Play => {
                        game_state.set(GameState::Game);
                        menu_state.set(MenuState::Disabled);
                    }
                    MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                    MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                }
            }
        }
    }
}

pub mod pause_menu {
    use bevy::{color, prelude::*};
    use crate::GameState;

    use crate::{SetFps, game::game_action, menus::main_menu::MenuState};

    /// This plugin manages the in-game Pause menu
    pub fn pause_menu_plugin(
        app: &mut App,
    ) {
        app
            .init_state::<InGameMenuState>()
            .add_systems(OnEnter(GameState::Paused), menu_setup)
            .add_systems(OnEnter(InGameMenuState::Base), in_game_menu_setup)
            .add_systems(Update, (menu_action, button_system, game_action).run_if(in_state(GameState::Paused)))
            .add_systems(OnEnter(InGameMenuState::Settings), in_game_settings_menu_setup)
            .add_systems(Update, setting_button::<SetFps>.run_if(in_state(InGameMenuState::Settings)));
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

    // This system updates the settings when a new value for a setting is selected, and marks
    // the button as the one currently selected
    fn setting_button<T: Resource + Component + PartialEq + Copy>(
        interaction_query: Query<(&Interaction, &T, Entity), (Changed<Interaction>, With<Button>)>,
        selected_query: Single<(Entity, &mut BackgroundColor), With<SelectedOption>>,
        mut commands: Commands,
        mut setting: ResMut<T>,
    ) {
        let (previous_button, mut previous_button_color) = selected_query.into_inner();
        for (interaction, button_setting, entity) in &interaction_query {
            if *interaction == Interaction::Pressed && *setting != *button_setting {
                *previous_button_color = NORMAL_BUTTON.into();
                commands.entity(previous_button).remove::<SelectedOption>();
                commands.entity(entity).insert(SelectedOption);
                *setting = *button_setting;
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
        let home_icon = asset_server.load(r"icons\home_icon.png");
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
                                Text::new("Resume Game"),
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
                        button_node.clone(),
                        BackgroundColor(NORMAL_BUTTON),
                        InGameMenuButtonAction::BackToMainMenu,
                        children![
                            (ImageNode::new(home_icon), button_icon_node.clone()),
                            (
                                Text::new("Back To Main Menu"),
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

    fn in_game_settings_menu_setup(
        mut commands: Commands,
        fps_limit: Res<SetFps>,
    ) {
        fn button_node() -> Node {
            Node {
                width: px(200),
                height: px(65),
                margin: UiRect::all(px(20)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            }
        }
        fn button_text_style() -> impl Bundle {
            (
                TextFont {
                    font_size: 33.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            )
        }

        let fps_limit = *fps_limit;
        commands.spawn((
            DespawnOnExit(InGameMenuState::Settings),
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(color::palettes::css::CRIMSON.into()),
            OnSettingsMenuScreen,
            children![(
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    // Creating a new `Node`, this time not setting its `flex_direction`
                    // Use the default value, `FlexDirection::Row`, from left to right.
                    (
                        Node {
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(color::palettes::css::CRIMSON.into()),
                        Children::spawn((
                            // Display a label for the current setting
                            Spawn((Text::new("FPS Limit"), button_text_style())),
                            SpawnWith(move |parent: &mut ChildSpawner| {
                                for fps_setting in [
                                    SetFps::Low,
                                    SetFps::Medium,
                                    SetFps::High,
                                    SetFps::Uncapped,
                                ] {
                                    let mut entity = parent.spawn((
                                        Button,
                                        Node {
                                            width: px(150),
                                            height: px(65),
                                            ..button_node()
                                        },
                                        BackgroundColor(NORMAL_BUTTON),
                                        fps_setting,
                                        children![(
                                            Text::new(format!("{fps_setting:?}")),
                                            button_text_style(),
                                        )],
                                    ));
                                    if fps_limit == fps_setting {
                                        entity.insert(SelectedOption);
                                    }
                                }
                            })
                        ))
                    ),
                    (
                        Button,
                        button_node(),
                        BackgroundColor(NORMAL_BUTTON),
                        InGameMenuButtonAction::BackToInGameMenu,
                        children![(Text::new("Back"), button_text_style())]
                    )
                ]
            )]
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
                    InGameMenuButtonAction::BackToMainMenu => {
                        game_state.set(GameState::Menu);
                        menu_state.set(MenuState::Main);
                        paused_menu_state.set(InGameMenuState::Disabled);
                    }
                }
            }
        }
    }
}
