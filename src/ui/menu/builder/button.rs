use super::*;
use bevy::{
    feathers::{
        controls::{ButtonVariant, FeathersButton},
        rounded_corners::RoundedCorners,
    },
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
        self.button_with_props(label, action, ButtonVariant::Normal, RoundedCorners::None)
    }

    pub fn button_with_props<M>(
        &mut self,
        label: impl Into<String>,
        action: impl WidgetObserver<Activate, M>,
        variant: ButtonVariant,
        corners: RoundedCorners,
    ) -> &mut Self
    where
        M: Send + Sync + 'static,
    {
        let root = self.root();
        let exit = self.current_open();
        let label = label.into();
        self.commands.spawn_scene(bsn! {
            @FeathersButton {
                @caption: bsn!((Text::new(label) ThemedText)),
                @variant: variant,
                @corners: corners,
            }
            ChildOf(root)
            DespawnOnExit::<MenuId>(exit)
            on(action)
        });
        // self.commands.spawn_scene((
        //     button(props, (), Spawn((Text::new(label), ThemedText))),
        //     observe(action),
        //     ChildOf(self.root()),
        //     DespawnOnExit(self.current_open()),
        // ));
        self
    }
}
