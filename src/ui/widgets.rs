use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    input::mouse::{AccumulatedMouseMotion, MouseMotion},
    prelude::*,
    ui_widgets::observe,
    window::{PrimaryWindow, WindowEvent},
};

pub struct WidgetPlugin;

impl Plugin for WidgetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, anchor_drag);
    }
}

#[derive(Component)]
#[component(on_add = Self::on_add)]
pub struct Anchor;

#[derive(Component)]
#[require(Node)]
pub struct AnchorHook {
    target: Entity,
    dragging: Option<(Vec2, f32, f32)>,
}

impl Anchor {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world.commands().entity(ctx.entity).with_child((
            AnchorHook {
                target: ctx.entity,
                dragging: None,
            },
            Node {
                top: Val::Px(2.),
                left: Val::Px(2.),
                position_type: PositionType::Absolute,
                min_width: Val::Px(16.),
                min_height: Val::Px(16.),
                ..Default::default()
            },
            BackgroundColor(Color::WHITE.darker(0.5)),
            observe(anchor_start_drag),
            observe(anchor_end_drag),
        ));
    }
}

fn anchor_drag(
    anchors: Query<&AnchorHook>,
    mut nodes: Query<&mut Node>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut total: Local<f32>,
) {
    let Some(pos) = window.cursor_position() else {
        return;
    };

    for hook in anchors.iter() {
        let Some((orgin, top, left)) = hook.dragging else {
            continue;
        };
        let delta = pos - orgin;
        let Ok(mut node) = nodes.get_mut(hook.target) else {
            warn!("Anchor target does not have a Node component");
            continue;
        };
        println!("Dragging anchor with delta: {delta}");
        node.top = Val::Px(pos.y - top);
        node.left = Val::Px(pos.x - left);
    }
}

fn anchor_start_drag(
    trigger: On<Pointer<DragStart>>,
    mut hooks: Query<&mut AnchorHook>,
    nodes: Query<&Node>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let Ok(mut hook) = hooks.get_mut(trigger.entity) else {
        return;
    };
    let Ok(node) = nodes.get(hook.target) else {
        return;
    };
    let Some(mut pos) = window.cursor_position() else {
        return;
    };

    let top = match node.top {
        Val::Auto => 0.,
        Val::Px(p) => p,
        Val::Percent(p) => p * window.height(),
        _ => {
            warn!("Unsupported Val type for top: {:?}", node.top);
            0.
        }
    };
    let left = match node.left {
        Val::Auto => 0.,
        Val::Px(p) => p,
        Val::Percent(p) => p * window.width(),
        _ => {
            warn!("Unsupported Val type for left: {:?}", node.left);
            0.
        }
    };

    pos.y -= top;
    pos.x -= left;

    hook.dragging = Some((trigger.hit.position.unwrap().truncate(), pos.y, pos.x));
}

fn anchor_end_drag(trigger: On<Pointer<DragEnd>>, mut hooks: Query<&mut AnchorHook>) {
    let Ok(mut hook) = hooks.get_mut(trigger.entity) else {
        return;
    };
    if hook.dragging.is_some() {
        println!("Anchor dropped");
    }
    hook.dragging = None;
}
