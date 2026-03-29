use std::hash::{Hash, Hasher};

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::system::{IntoObserverSystem, SystemId, SystemParam};
use bevy::feathers;
use bevy::input::common_conditions::input_just_pressed;
use bevy::{color::palettes::css::*, ecs::world::DeferredWorld, feathers::theme::ThemedText};
use bevy::{
    ui::Checked,
    ui_widgets::{SliderPrecision, SliderStep, SliderValue, ValueChange, observe},
};
use feathers::controls::*;

mod debug;
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

fn open_menu<M: Menu>() -> fn(ResMut<NextState<MenuId>>, Res<State<MenuId>>, ResMut<MenuStack>) {
    |mut next: ResMut<NextState<MenuId>>,
     current: Res<State<MenuId>>,
     mut stack: ResMut<MenuStack>| {
        stack.push(*current.get());
        next.set(M::id());
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

fn to_play(mut next: ResMut<NextState<GameState>>) {
    next.set(GameState::Playing);
}

fn clear_stack(mut stack: ResMut<MenuStack>) {
    stack.clear();
}

#[derive(SystemParam)]
pub struct MenuBuilder<'w, 's> {
    pub commands: Commands<'w, 's>,
    menu_state: MenuBuilderState,
}

struct MenuBuilderState {
    root: Entity,
    open: MenuId,
}

unsafe impl bevy::ecs::system::ReadOnlySystemParam for MenuBuilderState {}

unsafe impl SystemParam for MenuBuilderState {
    type Item<'world, 'state> = MenuBuilderState;
    type State = QueryState<Entity, With<MenuRoot>>;
    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        _system_meta: &bevy::ecs::system::SystemMeta,
        world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
        _change_tick: bevy::ecs::change_detection::Tick,
    ) -> Self::Item<'world, 'state> {
        let root = unsafe {
            state
                .query_unchecked(world)
                .single_inner()
                .expect("Should fail validation if no root is found, so this should never panic")
        };
        let open = unsafe {
            **world
                .get_resource::<State<MenuId>>()
                .expect("No State<MenuId> found")
        };

        Self { root, open }
    }

    fn init_state(world: &mut World) -> Self::State {
        QueryState::new(world)
    }

    fn init_access(
        state: &Self::State,
        system_meta: &mut bevy::ecs::system::SystemMeta,
        component_access_set: &mut bevy::ecs::query::FilteredAccessSet,
        world: &mut World,
    ) {
        Query::init_access(state, system_meta, component_access_set, world);
        let index = world.register_resource::<State<MenuId>>();
        component_access_set.add_unfiltered_resource_read(index);
    }

    #[inline]
    unsafe fn validate_param(
        state: &mut Self::State,
        _system_meta: &bevy::ecs::system::SystemMeta,
        world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell,
    ) -> Result<(), bevy::ecs::system::SystemParamValidationError> {
        // SAFETY: State ensures that the components it accesses are not mutably accessible elsewhere
        // and the query is read only.
        // The caller ensures the world matches the one used in init_state.
        let query = unsafe { state.query_unchecked(world) };
        match query.single_inner() {
            Ok(_) => Ok(()),
            Err(bevy::ecs::query::QuerySingleError::NoEntities(_)) => Err(
                bevy::ecs::system::SystemParamValidationError::skipped::<Self>("No MenuRoot Found"),
            ),
            Err(bevy::ecs::query::QuerySingleError::MultipleEntities(_)) => Err(
                bevy::ecs::system::SystemParamValidationError::skipped::<Self>(
                    "Multiple MenuRoots Found",
                ),
            ),
        }
    }
}

// helper functions for building menus
impl MenuBuilder<'_, '_> {
    pub fn root(&self) -> Entity {
        self.menu_state.root
    }

    pub fn current_open(&self) -> MenuId {
        self.menu_state.open
    }
}

// Checkbox implementation
impl MenuBuilder<'_, '_> {
    #[inline(always)]
    pub fn add_checkbox<B: Bundle, M, I>(
        &mut self,
        label: impl Into<String>,
        on_check: I,
    ) -> &mut Self
    where
        I: IntoObserverSystem<ValueChange<bool>, B, M> + Send + Sync + 'static,
        M: Send + Sync + 'static,
    {
        self.add_checkbox_with_state(label, on_check, false)
    }

    #[inline(always)]
    pub fn add_checkbox_with_state<B: Bundle, M, I>(
        &mut self,
        label: impl Into<String>,
        on_check: I,
        checked: bool,
    ) -> &mut Self
    where
        I: IntoObserverSystem<ValueChange<bool>, B, M> + Send + Sync + 'static,
        M: Send + Sync + 'static,
    {
        if checked {
            self.add_checkbox_with_ext(label, on_check, Checked)
        } else {
            self.add_checkbox_with_ext(label, on_check, ())
        }
    }

    pub fn add_checkbox_with_ext<B: Bundle, OB: Bundle, M, I>(
        &mut self,
        label: impl Into<String>,
        on_check: I,
        bundle: B,
    ) -> &mut Self
    where
        I: IntoObserverSystem<ValueChange<bool>, OB, M> + Send + Sync + 'static,
        M: Send + Sync + 'static,
    {
        let close = DespawnOnExit(self.current_open());
        self.commands.spawn((
            checkbox((), Spawn((Text::new(label), ThemedText))),
            observe(on_check),
            close,
            bundle,
            ChildOf(self.root()),
        ));
        self
    }
}

// Layout implementation
impl MenuBuilder<'_, '_> {
    pub fn horizontal(&mut self) -> MenuBuilder<'_, '_> {
        let new = self
            .commands
            .spawn((
                Node::DEFAULT,
                ChildOf(self.root()),
                DespawnOnExit(self.current_open()),
            ))
            .id();
        let state = MenuBuilderState {
            root: new,
            open: self.menu_state.open,
        };
        MenuBuilder {
            menu_state: state,
            commands: self.commands.reborrow(),
        }
    }
    pub fn vertical(&mut self) -> MenuBuilder<'_, '_> {
        let new = self
            .commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                ChildOf(self.root()),
                DespawnOnExit(self.current_open()),
            ))
            .id();
        let state = MenuBuilderState {
            root: new,
            open: self.menu_state.open,
        };
        MenuBuilder {
            menu_state: state,
            commands: self.commands.reborrow(),
        }
    }

    pub fn label(&mut self, text: impl Into<String>) -> &mut Self {
        self.commands.spawn((
            Text::new(text),
            ThemedText,
            ChildOf(self.root()),
            DespawnOnExit(self.current_open()),
        ));
        self
    }
}

#[derive(Debug)]
pub struct SliderSettings {
    min: f32,
    max: f32,
    current: f32,
    step: Option<f32>,
    precision: Option<i32>,
}

impl SliderSettings {
    pub fn new(min: f32, max: f32, current: f32) -> Self {
        Self {
            min,
            max,
            current,
            step: None,
            precision: None,
        }
    }
    pub fn with_step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }
    pub fn with_precision(mut self, precision: i32) -> Self {
        self.precision = Some(precision);
        self
    }
}

// Slider implementation
impl MenuBuilder<'_, '_> {
    #[inline(always)]
    pub fn add_slider<B: Bundle, M, I>(
        &mut self,
        on_change: I,
        settings: SliderSettings,
    ) -> &mut Self
    where
        I: IntoObserverSystem<ValueChange<f32>, B, M> + Send + Sync + 'static,
        M: Send + Sync + 'static,
    {
        self.add_slider_with_ext(on_change, settings, ())
    }
    pub fn add_slider_with_ext<B: Bundle, OB: Bundle, M, I>(
        &mut self,
        on_change: I,
        settings: SliderSettings,
        bundle: B,
    ) -> &mut Self
    where
        I: IntoObserverSystem<ValueChange<f32>, OB, M> + Send + Sync + 'static,
        M: Send + Sync + 'static,
    {
        let mut c = self.commands.spawn(slider(
            SliderProps {
                value: settings.current,
                min: settings.min,
                max: settings.max,
            },
            (
                observe(on_change),
                bundle,
                ChildOf(self.root()),
                SliderPrecision(settings.precision.unwrap_or(1)),
            ),
        ));
        if let Some(step) = settings.step {
            c.insert(SliderStep(step));
        }
        self
    }
}
