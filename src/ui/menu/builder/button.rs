use super::*;
use bevy::{
    feathers::controls::{ButtonProps, button},
    ui_widgets::Activate,
};

impl MenuBuilder<'_, '_> {
    #[inline(always)]
    pub fn button<M>(
        &mut self,
        label: impl Into<String>,
        action: impl WidgetObserver<Activate, M>,
    ) -> &mut Self
    where
        M: Send + Sync + 'static,
    {
        self.button_with_props(label, action, ButtonProps::default())
    }

    pub fn button_with_props<M>(
        &mut self,
        label: impl Into<String>,
        action: impl WidgetObserver<Activate, M>,
        props: ButtonProps,
    ) -> &mut Self
    where
        M: Send + Sync + 'static,
    {
        self.commands.spawn((
            button(props, (), Spawn((Text::new(label), ThemedText))),
            observe(action),
            ChildOf(self.root()),
            DespawnOnExit(self.current_open()),
        ));
        self
    }
}
