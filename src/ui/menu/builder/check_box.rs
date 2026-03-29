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
