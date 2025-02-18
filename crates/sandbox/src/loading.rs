//! A loading screen during which game assets are loaded.
//! This reduces stuttering, especially for audio on WASM.

use bevy::prelude::*;

use crate::{
    load_level::LevelResources,
    timer_trigger::{TimerFinished, TimerTrigger},
};

pub const LABEL_TEXT: Color = Color::srgb(0.867, 0.827, 0.412);

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>().add_sub_state::<Gameplay>(); // We set the substate up here.
    app.enable_state_scoped_entities::<Screen>();
    app.enable_state_scoped_entities::<Gameplay>();

    app.add_systems(OnEnter(Screen::Loading), spawn_loading_screen);
    app.add_systems(OnEnter(Gameplay::Transition), spawn_transition);

    app.add_systems(
        Update,
        continue_to_title_screen.run_if(in_state(Screen::Loading).and(all_assets_loaded)),
    );
    app.add_systems(
        Update,
        (
            tick_from_to_color,
            apply_from_to_color,
            apply_from_to_color_background,
            rotate,
        )
            .chain(),
    );
}

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum Screen {
    /// Loading required resources.
    #[default]
    Loading,
    Gameplay,
}

/// In this case, instead of deriving `States`, we derive `SubStates`
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
/// And we need to add an attribute to let us know what the source state is
/// and what value it needs to have. This will ensure that unless we're
/// in [`Screen::Gameplay`], the [`Gameplay`] state resource
/// will not exist.
#[source(Screen = Screen::Gameplay)]
pub enum Gameplay {
    #[default]
    /// Start the game, but hide the spawning (particles settling...)
    Transition,
    Running,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct LoadingMarker {
    id: String,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Rotate;

pub const ID_ROOT: &str = "root";
pub const ID_SPLASH: &str = "splash";

/// An extension trait for spawning UI containers.
pub trait Containers {
    /// Spawns a root node that covers the full screen
    /// and centers its content horizontally and vertically.
    fn ui_root(&mut self) -> EntityCommands;
}

impl Containers for Commands<'_, '_> {
    fn ui_root(&mut self) -> EntityCommands {
        self.spawn((
            Name::new("UI Root"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
    }
}

fn spawn_loading_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, StateScoped(Screen::Loading)));
    commands
        .ui_root()
        .insert((
            LoadingMarker { id: ID_ROOT.into() },
            BackgroundColor(Color::BLACK.to_srgba().into()),
        ))
        .with_children(|children| {
            children.spawn((
                Name::new("Splash image"),
                LoadingMarker {
                    id: ID_SPLASH.into(),
                },
                Node {
                    margin: UiRect::all(Val::Auto),
                    height: Val::Percent(50.0),
                    ..default()
                },
                ImageNode::new(asset_server.load("loading/donut.png")),
                Rotate,
            ));
        });
}

fn continue_to_title_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Gameplay);
}

fn all_assets_loaded(assets: Res<AssetServer>, resource_handles: Res<LevelResources>) -> bool {
    let LevelResources {
        map_def_handle,
        excavator_def,
        truck_def,
        bulldozer_model,
        excavator_model,
        truck_model,
        diffuse_map,
        specular_map,
    } = &*resource_handles;
    let handles = [
        &map_def_handle.clone_weak().untyped(),
        &excavator_def.clone_weak().untyped(),
        &truck_def.clone_weak().untyped(),
        &bulldozer_model.clone_weak().untyped(),
        &excavator_model.clone_weak().untyped(),
        &truck_model.clone_weak().untyped(),
        &diffuse_map.clone_weak().untyped(),
        &specular_map.clone_weak().untyped(),
    ];
    for h in handles {
        if !assets.is_loaded_with_dependencies(h) {
            return false;
        }
    }
    return true;
}

/// color fade in, Changes the state after delay, removes UI, color fade out.
fn spawn_transition(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Transition wait"),
            TimerTrigger(Timer::from_seconds(2f32, TimerMode::Once)),
        ))
        .insert(StateScoped(Gameplay::Transition))
        .observe(
            |trigger: Trigger<TimerFinished>,
             mut commands: Commands,
             q_loading: Query<(Entity, &LoadingMarker)>| {
                // Remove previous transition entity.
                commands.entity(trigger.entity()).despawn_recursive();
                // fade out
                for loading in q_loading.iter() {
                    commands.entity(loading.0).insert(FromToAlpha {
                        from: 1f32,
                        to: 0f32,
                        timer: Timer::from_seconds(1.0, TimerMode::Once),
                    });
                }
                commands
                    .spawn(TimerTrigger(Timer::from_seconds(1f32, TimerMode::Once)))
                    .insert(StateScoped(Gameplay::Transition))
                    .observe(
                        |trigger: Trigger<TimerFinished>,
                         mut commands: Commands,
                         q_loading: Query<(Entity, &LoadingMarker)>| {
                            // Remove loading UI
                            let e = q_loading
                                .iter()
                                .find(|(e, marker)| marker.id == ID_ROOT)
                                .unwrap()
                                .0;
                            commands.entity(e).despawn_recursive();
                            commands.entity(trigger.entity()).despawn_recursive();
                            commands.set_state(Gameplay::Running);
                        },
                    );
            },
        );
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FromToAlpha {
    pub from: f32,
    pub to: f32,
    pub timer: Timer,
}

fn tick_from_to_color(time: Res<Time>, mut animation_query: Query<&mut FromToAlpha>) {
    for mut anim in &mut animation_query {
        anim.timer.tick(time.delta());
    }
}

fn apply_from_to_color(mut to_update: Query<(&mut ImageNode, &FromToAlpha)>) {
    for (mut image, anim) in &mut to_update {
        image
            .color
            .set_alpha(anim.from.lerp(anim.to, anim.timer.fraction()));
    }
}

fn apply_from_to_color_background(
    mut to_update: Query<(&mut BackgroundColor, &FromToAlpha), Without<ImageNode>>,
) {
    for (mut color, anim) in to_update.iter_mut() {
        color
            .0
            .set_alpha(anim.from.lerp(anim.to, anim.timer.fraction()));
    }
}

fn rotate(mut to_update: Query<&mut Transform, With<Rotate>>) {
    for (mut transform) in to_update.iter_mut() {
        transform.rotate(Quat::from_rotation_z(90f32.to_radians() 
        // Estimate fps, using time would lead to big rotations if loading hangs
        * 1f32 / 60f32));
    }
}
