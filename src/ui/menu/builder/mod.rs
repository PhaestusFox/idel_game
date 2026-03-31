use bevy::{
    ecs::system::{IntoObserverSystem, SystemParam},
    feathers::theme::ThemedText,
    prelude::*,
    ui_widgets::observe,
};

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
}

pub mod button;
pub mod check_box;
pub mod label;
pub mod slider;
pub use slider::SliderSettings;

use crate::ui::{MenuId, menu::MenuRoot};

pub trait WidgetObserver<E: Event, M: Send + Sync + 'static>:
    IntoObserverSystem<E, (), M> + Send + Sync + 'static
{
}

impl<A, E, M> WidgetObserver<E, M> for A
where
    A: IntoObserverSystem<E, (), M> + Send + Sync + 'static,
    M: Send + Sync + 'static,
    E: Event,
{
}
