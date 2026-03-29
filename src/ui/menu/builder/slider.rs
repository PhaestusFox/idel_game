use bevy::{
    feathers::controls::{SliderProps, slider},
    ui_widgets::{SliderPrecision, SliderStep, ValueChange},
};

use super::*;

#[derive(Debug)]
pub struct SliderSettings {
    min: f32,
    max: f32,
    current: f32,
    step: Option<f32>,
    precision: Option<i32>,
}

impl SliderSettings {
    pub fn new(min: f32, max: f32, current: f32) -> Self {
        Self {
            min,
            max,
            current,
            step: None,
            precision: None,
        }
    }
    pub fn with_step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }
    pub fn with_precision(mut self, precision: i32) -> Self {
        self.precision = Some(precision);
        self
    }
}

// Slider implementation
impl MenuBuilder<'_, '_> {
    #[inline(always)]
    pub fn add_slider<B: Bundle, M, I>(
        &mut self,
        on_change: I,
        settings: SliderSettings,
    ) -> &mut Self
    where
        I: IntoObserverSystem<ValueChange<f32>, B, M> + Send + Sync + 'static,
        M: Send + Sync + 'static,
    {
        self.add_slider_with_ext(on_change, settings, ())
    }
    pub fn add_slider_with_ext<B: Bundle, OB: Bundle, M, I>(
        &mut self,
        on_change: I,
        settings: SliderSettings,
        bundle: B,
    ) -> &mut Self
    where
        I: IntoObserverSystem<ValueChange<f32>, OB, M> + Send + Sync + 'static,
        M: Send + Sync + 'static,
    {
        let mut c = self.commands.spawn(slider(
            SliderProps {
                value: settings.current,
                min: settings.min,
                max: settings.max,
            },
            (
                observe(on_change),
                bundle,
                ChildOf(self.root()),
                SliderPrecision(settings.precision.unwrap_or(1)),
            ),
        ));
        if let Some(step) = settings.step {
            c.insert(SliderStep(step));
        }
        self
    }
}
