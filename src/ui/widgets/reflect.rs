use bevy::{ecs::{lifecycle::HookContext, world::DeferredWorld}, prelude::*};

mod curve;


pub struct ReflectWidgets;

impl Plugin for ReflectWidgets {
    fn build(&self, app: &mut App) {
        // app.register_type::<EasingCurve<f32>>();
        let mut registry = app.world_mut().resource::<AppTypeRegistry>().write();
        registry.register_type_data::<EasingCurve<f32>, ReflectWidget>();
    }
}

#[derive(Component, Deref)]
#[require(Node)]
#[component(immutable, on_add = Self::on_add, on_insert = Self::on_insert)]
pub struct ReflectWidgetRoot(pub Box<dyn Reflect>);

impl ReflectWidgetRoot {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let w2 = world.as_unsafe_world_cell();
        let world = unsafe {
            w2.clone().world()};
        let app_registry = world.resource::<AppTypeRegistry>();
        let app_registry = app_registry.read();
        let item = world.get::<ReflectWidgetRoot>(ctx.entity).unwrap();
        let Some(f) = app_registry.get(item.0.type_id()) else {
            error!("Failed to find type in registry for {:?}", item.as_partial_reflect().get_represented_type_info());
            return;
        };
        let Some(widget) = f.data::<ReflectWidget>() else {
            error!("{:?} dose not Reflect Widget", item.as_partial_reflect().get_represented_type_info().unwrap().ty());
            return;
        };
        let Some(w) = widget.get(item.0.as_reflect()) else {
            error!("Failed to downcast {:?} to ReflectWidget", item.as_partial_reflect().get_represented_type_info().unwrap().ty());
            return;
        };
        w.spawn(unsafe {
            w2.into_deferred()
        }, ctx.entity);
    }

    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let w2 = world.as_unsafe_world_cell();
        let world = unsafe {
            w2.clone().world()};
        let app_registry = world.resource::<AppTypeRegistry>();
        let app_registry = app_registry.read();
        let item = world.get::<ReflectWidgetRoot>(ctx.entity).unwrap();
        let Some(f) = app_registry.get(item.0.type_id()) else {
            error!("Failed to find type in registry for {:?}", item.as_partial_reflect().get_represented_type_info());
            return;
        };
        let Some(widget) = f.data::<ReflectWidget>() else {
            error!("{:?} dose not Reflect Widget", item.as_partial_reflect().get_represented_type_info().unwrap().ty());
            return;
        };
        let Some(w) = widget.get(item.0.as_reflect()) else {
            error!("Failed to downcast {:?} to ReflectWidget", item.as_partial_reflect().get_represented_type_info().unwrap().ty());
            return;
        };
        w.update(unsafe {
            w2.into_deferred()
        }, ctx.entity);
    }
}

#[reflect_trait]
pub trait Widget {
    fn spawn(&self, world: DeferredWorld, root: Entity);
    fn update(&self, world: DeferredWorld, root: Entity);
}