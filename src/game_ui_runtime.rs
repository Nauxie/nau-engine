use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::camera_runtime::MouseLookState;
use crate::power_up_runtime::PowerUpCollectionState;

const HUD_BACKGROUND: Color = Color::srgba(0.025, 0.035, 0.055, 0.82);
const MENU_BACKGROUND: Color = Color::srgba(0.018, 0.025, 0.04, 0.94);
const MENU_OVERLAY: Color = Color::srgba(0.01, 0.015, 0.025, 0.66);
const BUTTON_BACKGROUND: Color = Color::srgb(0.10, 0.15, 0.20);
const BUTTON_HOVERED: Color = Color::srgb(0.15, 0.23, 0.29);
const BUTTON_PRESSED: Color = Color::srgb(0.20, 0.34, 0.39);
const TEXT_PRIMARY: Color = Color::srgb(0.93, 0.96, 0.97);
const TEXT_MUTED: Color = Color::srgb(0.62, 0.70, 0.74);
const ACCENT: Color = Color::srgb(0.34, 0.84, 0.78);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum GameMenuScreen {
    #[default]
    Closed,
    Pause,
    Controls,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GameUiState {
    enabled: bool,
    screen: GameMenuScreen,
    capture_on_resume: bool,
}

impl GameUiState {
    pub(crate) const fn new(enabled: bool) -> Self {
        Self {
            enabled,
            screen: GameMenuScreen::Closed,
            capture_on_resume: false,
        }
    }

    fn handle_escape(&mut self, mouse_captured: bool) {
        if self.screen == GameMenuScreen::Closed {
            self.capture_on_resume = mouse_captured;
        }
        self.screen = match self.screen {
            GameMenuScreen::Closed => GameMenuScreen::Pause,
            GameMenuScreen::Pause => GameMenuScreen::Closed,
            GameMenuScreen::Controls => GameMenuScreen::Pause,
        };
    }
}

#[derive(Component, Debug)]
pub(crate) struct GameHudRoot;

#[derive(Component, Debug)]
pub(crate) struct CheckpointScoreText;

#[derive(Component, Debug)]
pub(crate) struct GameMenuOverlay;

#[derive(Component, Debug)]
pub(crate) struct PauseMenuPanel;

#[derive(Component, Debug)]
pub(crate) struct ControlsMenuPanel;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GameMenuAction {
    Resume,
    Controls,
    Back,
    Quit,
}

type GameMenuButtonInteractions<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        &'static GameMenuAction,
        &'static mut BackgroundColor,
    ),
    (Changed<Interaction>, With<Button>),
>;

pub(crate) fn gameplay_input_active(state: Res<GameUiState>) -> bool {
    !state.enabled || state.screen == GameMenuScreen::Closed
}

pub(crate) fn toggle_game_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<GameUiState>,
    mut time: ResMut<Time<Virtual>>,
    mut mouse_look: ResMut<MouseLookState>,
    mut window: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    if !state.enabled {
        mouse_look.captured = false;
        if let Ok(mut cursor) = window.single_mut() {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        }
        return;
    }

    state.handle_escape(mouse_look.captured);
    apply_menu_capture_state(&state, &mut time, &mut mouse_look, &mut window);
}

pub(crate) fn handle_game_menu_buttons(
    mut interactions: GameMenuButtonInteractions,
    mut state: ResMut<GameUiState>,
    mut time: ResMut<Time<Virtual>>,
    mut mouse_look: ResMut<MouseLookState>,
    mut mouse_buttons: ResMut<ButtonInput<MouseButton>>,
    mut window: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if !state.enabled {
        return;
    }

    let mut screen_changed = false;
    for (interaction, action, mut background) in &mut interactions {
        background.0 = match interaction {
            Interaction::Pressed => BUTTON_PRESSED,
            Interaction::Hovered => BUTTON_HOVERED,
            Interaction::None => BUTTON_BACKGROUND,
        };

        if *interaction != Interaction::Pressed {
            continue;
        }

        if apply_game_menu_action(&mut state, *action) {
            app_exit.write(AppExit::Success);
        } else {
            if state.screen == GameMenuScreen::Closed {
                mouse_buttons.clear_just_pressed(MouseButton::Left);
            }
            screen_changed = true;
        }
    }

    if screen_changed {
        apply_menu_capture_state(&state, &mut time, &mut mouse_look, &mut window);
    }
}

fn apply_game_menu_action(state: &mut GameUiState, action: GameMenuAction) -> bool {
    state.screen = match action {
        GameMenuAction::Resume => GameMenuScreen::Closed,
        GameMenuAction::Controls => GameMenuScreen::Controls,
        GameMenuAction::Back => GameMenuScreen::Pause,
        GameMenuAction::Quit => return true,
    };
    false
}

fn apply_menu_capture_state(
    state: &GameUiState,
    time: &mut Time<Virtual>,
    mouse_look: &mut MouseLookState,
    window: &mut Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if state.screen == GameMenuScreen::Closed {
        time.unpause();
        mouse_look.captured = state.capture_on_resume;
    } else {
        time.pause();
        mouse_look.captured = false;
    }

    if let Ok(mut cursor) = window.single_mut() {
        cursor.grab_mode = if mouse_look.captured {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        cursor.visible = !mouse_look.captured;
    }
}

pub(crate) fn sync_game_ui(
    mut commands: Commands,
    state: Res<GameUiState>,
    collection: Res<PowerUpCollectionState>,
    mut last_collected_count: Local<Option<usize>>,
    mut rendered_screen: Local<Option<GameMenuScreen>>,
    mut score_text: Query<&mut Text, With<CheckpointScoreText>>,
    overlay: Query<Entity, With<GameMenuOverlay>>,
) {
    if !state.enabled {
        return;
    }

    let collected_count = collection.collected_count();
    if *last_collected_count != Some(collected_count) {
        let score = checkpoint_score_label(&collection);
        for mut text in &mut score_text {
            **text = score.clone();
        }
        *last_collected_count = Some(collected_count);
    }

    if *rendered_screen == Some(state.screen) {
        return;
    }

    for entity in &overlay {
        commands.entity(entity).despawn();
    }
    if state.screen != GameMenuScreen::Closed {
        spawn_game_menu_overlay(&mut commands, state.screen, &collection);
    }
    *rendered_screen = Some(state.screen);
}

fn checkpoint_score_label(collection: &PowerUpCollectionState) -> String {
    format!(
        "AIR GATES  {} / {}",
        collection.collected_count(),
        collection.total_count()
    )
}

fn empty_checkpoint_score_label() -> String {
    checkpoint_score_label(&PowerUpCollectionState::default())
}

pub(crate) fn spawn_game_ui(commands: &mut Commands, state: &GameUiState) {
    if !state.enabled {
        return;
    }

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(18.0),
                top: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: Val::Px(3.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(HUD_BACKGROUND),
            ZIndex(20),
            GameHudRoot,
        ))
        .with_children(|hud| {
            hud.spawn((
                Text::new(empty_checkpoint_score_label()),
                TextFont {
                    font_size: 17.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
                CheckpointScoreText,
            ));
            hud.spawn((
                Text::new("ESC  MENU"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(TEXT_MUTED),
            ));
        });
}

fn spawn_game_menu_overlay(
    commands: &mut Commands,
    screen: GameMenuScreen,
    collection: &PowerUpCollectionState,
) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(MENU_OVERLAY),
            ZIndex(100),
            GameMenuOverlay,
        ))
        .with_children(|overlay| match screen {
            GameMenuScreen::Pause => {
                overlay
                    .spawn((
                        menu_panel_node(),
                        BackgroundColor(MENU_BACKGROUND),
                        PauseMenuPanel,
                    ))
                    .with_children(|panel| {
                        panel.spawn(menu_title("NAU"));
                        panel.spawn(menu_subtitle("FLIGHT SANDBOX"));
                        panel.spawn((
                            Text::new(checkpoint_score_label(collection)),
                            TextFont {
                                font_size: 15.0,
                                ..default()
                            },
                            TextColor(ACCENT),
                            CheckpointScoreText,
                        ));
                        panel.spawn(menu_button("RESUME", GameMenuAction::Resume));
                        panel.spawn(menu_button("CONTROLS", GameMenuAction::Controls));
                        panel.spawn(menu_button("QUIT", GameMenuAction::Quit));
                    });
            }
            GameMenuScreen::Controls => {
                overlay
                    .spawn((
                        menu_panel_node(),
                        BackgroundColor(MENU_BACKGROUND),
                        ControlsMenuPanel,
                    ))
                    .with_children(|panel| {
                        panel.spawn(menu_title("CONTROLS"));
                        panel.spawn(control_row("WASD", "Move / steer"));
                        panel.spawn(control_row("MOUSE", "Look when locked"));
                        panel.spawn(control_row("LEFT CLICK", "Lock cursor"));
                        panel.spawn(control_row("RIGHT MOUSE", "Hold to look"));
                        panel.spawn(control_row("SPACE", "Deploy glider"));
                        panel.spawn(control_row("SHIFT", "Dive"));
                        panel.spawn(control_row("E", "Launch"));
                        panel.spawn(control_row("R", "Reset to main island"));
                        panel.spawn(control_row("ESC", "Menu / back"));
                        panel.spawn(menu_button("BACK", GameMenuAction::Back));
                    });
            }
            GameMenuScreen::Closed => {}
        });
}

fn menu_panel_node() -> Node {
    Node {
        width: Val::Px(340.0),
        max_width: Val::Percent(90.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        row_gap: Val::Px(10.0),
        padding: UiRect::all(Val::Px(22.0)),
        border_radius: BorderRadius::all(Val::Px(6.0)),
        ..default()
    }
}

fn menu_title(label: &'static str) -> impl Bundle {
    (
        Text::new(label),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(TEXT_PRIMARY),
    )
}

fn menu_subtitle(label: &'static str) -> impl Bundle {
    (
        Text::new(label),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        TextColor(TEXT_MUTED),
    )
}

fn menu_button(label: &'static str, action: GameMenuAction) -> impl Bundle {
    (
        Button,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(38.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(5.0)),
            ..default()
        },
        BackgroundColor(BUTTON_BACKGROUND),
        action,
        children![(
            Text::new(label),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(TEXT_PRIMARY),
        )],
    )
}

fn control_row(key: &'static str, action: &'static str) -> impl Bundle {
    (
        Text::new(format!("{key}  |  {action}")),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(TEXT_MUTED),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::window::PrimaryWindow;

    #[test]
    fn escape_moves_between_play_pause_and_controls() {
        let mut state = GameUiState::new(true);

        state.handle_escape(false);
        assert_eq!(state.screen, GameMenuScreen::Pause);

        state.screen = GameMenuScreen::Controls;
        state.handle_escape(false);
        assert_eq!(state.screen, GameMenuScreen::Pause);

        state.handle_escape(false);
        assert_eq!(state.screen, GameMenuScreen::Closed);
    }

    #[test]
    fn checkpoint_label_uses_authoritative_collection_total() {
        let collection = PowerUpCollectionState::default();

        assert_eq!(
            checkpoint_score_label(&collection),
            format!("AIR GATES  0 / {}", collection.total_count())
        );
    }

    #[test]
    fn menu_actions_navigate_without_conflating_quit() {
        let mut state = GameUiState::new(true);
        state.screen = GameMenuScreen::Pause;

        assert!(!apply_game_menu_action(
            &mut state,
            GameMenuAction::Controls
        ));
        assert_eq!(state.screen, GameMenuScreen::Controls);
        assert!(!apply_game_menu_action(&mut state, GameMenuAction::Back));
        assert_eq!(state.screen, GameMenuScreen::Pause);
        assert!(!apply_game_menu_action(&mut state, GameMenuAction::Resume));
        assert_eq!(state.screen, GameMenuScreen::Closed);
        assert!(apply_game_menu_action(&mut state, GameMenuAction::Quit));
        assert_eq!(state.screen, GameMenuScreen::Closed);
    }

    #[test]
    fn resume_click_does_not_recapture_an_unlocked_cursor() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(GameUiState {
                enabled: true,
                screen: GameMenuScreen::Pause,
                capture_on_resume: false,
            })
            .insert_resource(MouseLookState::default())
            .insert_resource(ButtonInput::<MouseButton>::default())
            .add_systems(Update, handle_game_menu_buttons);
        app.world_mut()
            .spawn((Window::default(), CursorOptions::default(), PrimaryWindow));
        app.world_mut().spawn((
            Button,
            Interaction::Pressed,
            BackgroundColor(BUTTON_BACKGROUND),
            GameMenuAction::Resume,
        ));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);

        app.update();

        assert_eq!(
            app.world().resource::<GameUiState>().screen,
            GameMenuScreen::Closed
        );
        assert!(
            !app.world()
                .resource::<ButtonInput<MouseButton>>()
                .just_pressed(MouseButton::Left)
        );
        assert!(!app.world().resource::<MouseLookState>().captured);
    }

    #[test]
    fn escape_pauses_time_and_releases_the_cursor() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(GameUiState::new(true))
            .insert_resource(MouseLookState { captured: true })
            .insert_resource(ButtonInput::<KeyCode>::default())
            .add_systems(Update, toggle_game_menu);
        app.world_mut().spawn((
            Window::default(),
            CursorOptions {
                grab_mode: CursorGrabMode::Locked,
                visible: false,
                ..default()
            },
            PrimaryWindow,
        ));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);

        app.update();

        assert_eq!(
            app.world().resource::<GameUiState>().screen,
            GameMenuScreen::Pause
        );
        assert!(app.world().resource::<Time<Virtual>>().is_paused());
        assert!(!app.world().resource::<MouseLookState>().captured);
        let cursor = app
            .world_mut()
            .query_filtered::<&CursorOptions, With<PrimaryWindow>>()
            .single(app.world())
            .expect("primary window should exist");
        assert_eq!(cursor.grab_mode, CursorGrabMode::None);
        assert!(cursor.visible);
    }

    #[test]
    fn escape_resume_restores_prior_mouse_capture() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(GameUiState::new(true))
            .insert_resource(MouseLookState { captured: true })
            .insert_resource(ButtonInput::<KeyCode>::default())
            .add_systems(Update, toggle_game_menu);
        app.world_mut().spawn((
            Window::default(),
            CursorOptions {
                grab_mode: CursorGrabMode::Locked,
                visible: false,
                ..default()
            },
            PrimaryWindow,
        ));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);

        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset(KeyCode::Escape);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();

        assert_eq!(
            app.world().resource::<GameUiState>().screen,
            GameMenuScreen::Closed
        );
        assert!(!app.world().resource::<Time<Virtual>>().is_paused());
        assert!(app.world().resource::<MouseLookState>().captured);
        let cursor = app
            .world_mut()
            .query_filtered::<&CursorOptions, With<PrimaryWindow>>()
            .single(app.world())
            .expect("primary window should exist");
        assert_eq!(cursor.grab_mode, CursorGrabMode::Locked);
        assert!(!cursor.visible);
    }

    #[test]
    fn disabled_menu_escape_only_releases_cursor_capture() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(GameUiState::new(false))
            .insert_resource(MouseLookState { captured: true })
            .insert_resource(ButtonInput::<KeyCode>::default())
            .add_systems(Update, toggle_game_menu);
        app.world_mut().spawn((
            Window::default(),
            CursorOptions {
                grab_mode: CursorGrabMode::Locked,
                visible: false,
                ..default()
            },
            PrimaryWindow,
        ));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);

        app.update();

        assert_eq!(
            app.world().resource::<GameUiState>().screen,
            GameMenuScreen::Closed
        );
        assert!(!app.world().resource::<Time<Virtual>>().is_paused());
        assert!(!app.world().resource::<MouseLookState>().captured);
        let cursor = app
            .world_mut()
            .query_filtered::<&CursorOptions, With<PrimaryWindow>>()
            .single(app.world())
            .expect("primary window should exist");
        assert_eq!(cursor.grab_mode, CursorGrabMode::None);
        assert!(cursor.visible);
    }

    #[test]
    fn menu_tree_exists_only_for_the_active_screen() {
        let mut app = App::new();
        app.insert_resource(GameUiState::new(true))
            .insert_resource(PowerUpCollectionState::default())
            .add_systems(Update, sync_game_ui);

        app.update();
        assert_eq!(component_count::<GameMenuOverlay>(app.world_mut()), 0);

        app.world_mut().resource_mut::<GameUiState>().screen = GameMenuScreen::Pause;
        app.update();
        assert_eq!(component_count::<GameMenuOverlay>(app.world_mut()), 1);
        assert_eq!(component_count::<PauseMenuPanel>(app.world_mut()), 1);
        assert_eq!(component_count::<ControlsMenuPanel>(app.world_mut()), 0);

        app.world_mut().resource_mut::<GameUiState>().screen = GameMenuScreen::Controls;
        app.update();
        assert_eq!(component_count::<GameMenuOverlay>(app.world_mut()), 1);
        assert_eq!(component_count::<PauseMenuPanel>(app.world_mut()), 0);
        assert_eq!(component_count::<ControlsMenuPanel>(app.world_mut()), 1);
        assert_eq!(live_entity_count(app.world_mut()), 14);

        app.world_mut().resource_mut::<GameUiState>().screen = GameMenuScreen::Closed;
        app.update();
        assert_eq!(component_count::<GameMenuOverlay>(app.world_mut()), 0);
    }

    fn component_count<T: Component>(world: &mut World) -> usize {
        let mut query = world.query_filtered::<Entity, With<T>>();
        query.iter(world).count()
    }

    fn live_entity_count(world: &mut World) -> usize {
        let mut query = world.query::<Entity>();
        query.iter(world).count()
    }
}
