use std::hash::{Hash, Hasher};

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::system::{SystemId, SystemParam};
use bevy::feathers;
use bevy::input::common_conditions::input_just_pressed;
use bevy::{color::palettes::css::*, ecs::world::DeferredWorld, feathers::theme::ThemedText};
use bevy::{
    ui::Checked,
    ui_widgets::{SliderValue, ValueChange},
};
use feathers::controls::*;

pub mod debug;
mod esc_menu;
mod main;
mod settings_menu;

use bevy::{
    ecs::{schedule::ScheduleConfigs, system::ScheduleSystem},
    prelude::*,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_sub_state::<MenuId>()
            .add_menu::<main::MainMenu>()
            .add_menu::<esc_menu::EscapeMenu>()
            .add_menu::<settings_menu::SettingsMenu>()
            .add_menu::<NoMenu>()
            .init_resource::<MenuStack>()
            .add_menu::<debug::DebugMenu>()
            .add_observer(on_press);
        app.add_systems(
            OnEnter(GameState::InMenu),
            (spawn_blank_menu, crate::player::show_cursor),
        );
        app.add_systems(Update, on_esc.run_if(input_just_pressed(KeyCode::Escape)));
    }
}

pub trait Menu {
    const MENU_ID: MenuId;
    fn init(_app: &mut App) {}
    fn open() -> ScheduleConfigs<ScheduleSystem>;
    fn close() -> Option<ScheduleConfigs<ScheduleSystem>> {
        None
    }
    fn id() -> MenuId {
        Self::MENU_ID
    }
}

pub trait AppMenuExt {
    fn add_menu<M: Menu>(&mut self) -> &mut Self;
}

impl AppMenuExt for App {
    fn add_menu<M: Menu>(&mut self) -> &mut Self {
        M::init(self);
        self.add_systems(OnEnter(M::id()), M::open());
        if let Some(close_schedule) = M::close() {
            self.add_systems(OnExit(M::id()), close_schedule);
        }
        self
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
struct MenuStack(Vec<MenuId>);

impl PartialEq for MenuId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for MenuId {}

impl Hash for MenuId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}

impl Default for MenuId {
    fn default() -> Self {
        main::MainMenu::id()
    }
}

#[derive(Component)]
#[component(on_remove = Self::drop_id)]
struct MenuAction {
    run: SystemId,
    owned: bool,
}

impl MenuAction {
    fn new(system: SystemId) -> Self {
        Self {
            run: system,
            owned: false,
        }
    }

    fn from_commands<M, S: 'static + IntoSystem<(), (), M>>(
        commands: &mut Commands,
        system: S,
    ) -> Self {
        let system_id = commands.register_system(system);
        Self {
            run: system_id,
            owned: true,
        }
    }

    fn drop_id(mut world: DeferredWorld, ctx: HookContext) {
        let Some(action) = world.get::<Self>(ctx.entity) else {
            return;
        };
        if action.owned {
            let run = action.run;
            world.commands().unregister_system(run);
        }
    }
}

use bevy::ui_widgets::Activate;

use crate::GameState;

fn on_press(click: On<Activate>, actions: Query<&MenuAction>, mut commands: Commands) {
    if let Ok(action) = actions.get(click.entity) {
        commands.run_system(action.run);
    }
}

#[derive(Resource, Default)]
struct NoMenu;
impl Menu for NoMenu {
    const MENU_ID: MenuId = MenuId(0);
    fn open() -> ScheduleConfigs<ScheduleSystem> {
        (crate::player::hide_cursor).into_configs()
    }
    fn close() -> Option<ScheduleConfigs<ScheduleSystem>> {
        Some((crate::player::show_cursor).into_configs())
    }
}

#[derive(SubStates, Debug, Clone, Copy, Deref, DerefMut)]
#[source(GameState = GameState::InMenu)]
pub struct MenuId(u64);

impl MenuId {
    #[inline]
    pub const fn new(ident: &'static str) -> Self {
        let hasher = const_siphasher::prelude::sip::SipHasher::new();
        Self(hasher.hash(ident.as_bytes()) << 1)
    }

    #[inline(always)]
    pub const fn new_auto_close(ident: &'static str) -> Self {
        let mut id = Self::new(ident).0;
        id |= 1;
        Self(id)
    }

    pub const fn auto_close(self) -> bool {
        self.0 & 1 == 1
    }
}

#[derive(Component)]
struct MenuRoot;

fn spawn_blank_menu(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(GameState::InMenu),
        Name::new("MenuRoot"),
        MenuRoot,
        Node {
            width: Val::Percent(30.0),
            height: Val::Percent(75.0),
            justify_content: JustifyContent::SpaceEvenly,
            padding: UiRect::all(Val::Px(16.0)),
            margin: UiRect::all(Val::Auto),
            flex_direction: FlexDirection::Column,
            border_radius: BorderRadius::all(Val::Percent(5.)),
            ..default()
        },
        BackgroundColor(Color::from(GRAY).with_alpha(0.8)),
    ));
}

fn on_esc(
    state: Res<State<GameState>>,
    open_menu: Option<Res<State<MenuId>>>,
    mut next_menu: ResMut<NextState<MenuId>>,
    mut stack: ResMut<MenuStack>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    match state.get() {
        GameState::InMenu => {
            if let Some(prev) = stack.pop() {
                next_menu.set(prev);
            } else if let Some(open) = open_menu {
                if open.auto_close() {
                    next_state.set(GameState::Playing);
                }
            } else {
                warn!("Pressed escape while in menu, but no menu was open");
                next_state.set(GameState::Playing);
            }
        }
        GameState::Playing => {
            next_state.set(GameState::InMenu);
            if stack.is_empty() {
                next_menu.set(esc_menu::EscapeMenu::id());
            } else {
                next_menu.set(stack.last().copied().expect("We know len > 0"));
            }
        }
    }
}

fn to_play(_: On<Activate>, mut state: ResMut<NextState<GameState>>) {
    state.set(GameState::Playing);
}

fn clear_stack(mut stack: ResMut<MenuStack>) {
    stack.clear();
}

mod builder;
pub use builder::MenuBuilder;

fn open_menu<M: Menu>()
-> fn(On<Activate>, ResMut<NextState<MenuId>>, Res<State<MenuId>>, ResMut<MenuStack>) {
    |_: On<Activate>,
     mut state: ResMut<NextState<MenuId>>,
     current: Res<State<MenuId>>,
     mut stack: ResMut<MenuStack>| {
        stack.push(*current.get());
        state.set(M::id());
    }
}

fn quit(_: On<Activate>, mut quit: MessageWriter<AppExit>) {
    quit.write(AppExit::Success);
}
