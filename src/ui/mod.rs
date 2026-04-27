mod menu;

pub use menu::*;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // menu
        app.add_plugins(bevy::feathers::FeathersPlugins);
        app.insert_resource(bevy::feathers::theme::UiTheme(
            bevy::feathers::dark_theme::create_dark_theme(),
        ));
        app.add_plugins(MenuPlugin);

        // widgets
        app.add_plugins(WidgetPlugin);

        app.add_systems(
            Update,
            spawn_test_widget.run_if(bevy::input::common_conditions::input_just_pressed(
                KeyCode::F11,
            )),
        );
        app.add_systems(
            Update,
            update_test_widget.run_if(bevy::input::common_conditions::input_just_pressed(
                KeyCode::F12,
            )),
        );
    }
}

mod widgets;
pub use menu::debug::DebugConsole;
pub use widgets::*;

fn spawn_test_widget(mut commands: Commands) {
    let c: EasingCurve<f32> = EasingCurve::new(0., 1., EaseFunction::ElasticOut);
    commands.spawn((
        Name::new("Test Widget"),
        widgets::ReflectWidgetRoot(Box::new(c)),
    ));
}

fn update_test_widget(query: Query<(Entity, &ReflectWidgetRoot)>, mut commands: Commands) {
    for (root, _) in query.iter() {
        let c: EasingCurve<f32> = EasingCurve::new(0., 1., EaseFunction::ExponentialInOut);
        commands.entity(root).insert(ReflectWidgetRoot(Box::new(c)));
    }
}

pub use menu::builder::SliderSettings;
