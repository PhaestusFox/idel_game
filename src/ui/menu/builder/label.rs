use super::*;

impl MenuBuilder<'_, '_> {
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
