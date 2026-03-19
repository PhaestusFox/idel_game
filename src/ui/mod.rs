mod menu;

pub use menu::*;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        /// menu
        app.add_plugins(bevy::feathers::FeathersPlugins);
        app.insert_resource(bevy::feathers::theme::UiTheme(
            bevy::feathers::dark_theme::create_dark_theme(),
        ));
        app.add_plugins(MenuPlugin);

        /// widgets
        app.add_plugins(WidgetPlugin);

        app.add_systems(
            Update,
            spawn_test_widget.run_if(bevy::input::common_conditions::input_just_pressed(
                KeyCode::F11,
            )),
        );
    }
}

mod widgets;
pub use widgets::*;

fn spawn_test_widget(mut commands: Commands) {
    commands.spawn((
        Name::new("Test Widget"),
        Node {
            width: Val::Px(200.),
            height: Val::Px(200.),
            position_type: PositionType::Absolute,
            ..Default::default()
        },
        BackgroundColor(Color::WHITE),
        Anchor,
    ));
}
