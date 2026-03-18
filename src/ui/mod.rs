mod menu;

pub use menu::*;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy::feathers::FeathersPlugins);
        app.insert_resource(bevy::feathers::theme::UiTheme(
            bevy::feathers::dark_theme::create_dark_theme(),
        ));
        app.add_plugins(MenuPlugin);
    }
}
