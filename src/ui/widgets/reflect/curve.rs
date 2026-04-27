use core::f32;

use bevy::{log::tracing_subscriber::fmt::format, render::render_resource::{Extent3d, TextureDimension, TextureFormat}};

use super::*;
use bevy::math::Curve;

const CURVE_SAMPLES: u32 = 1000;

impl Widget for EasingCurve<f32> {
    fn spawn(&self, mut world: DeferredWorld, root: Entity) {
        let mut node = world.get_mut::<Node>(root).unwrap();
        node.width = Val::Px(500.);
        node.height = Val::Px(500.);
        node.flex_wrap = FlexWrap::Wrap;
        let mut image = Image::new_fill(Extent3d {
            width: CURVE_SAMPLES,
           height: CURVE_SAMPLES,
           depth_or_array_layers: 1, 
        }, TextureDimension::D2, [(1f32).to_le_bytes();4].as_flattened(), TextureFormat::Rgba32Float, Default::default());

        let x_step = self.domain().length() / CURVE_SAMPLES as f32;
        let x_start = self.domain().start();
        let mut res = Vec::with_capacity(CURVE_SAMPLES as usize);
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for x in 0..CURVE_SAMPLES {
            let sx = x_start + x as f32 * x_step;
            let r = self.sample_unchecked(sx);
            res.push(r);
            min = min.min(r);
            max = max.max(r);
        }
        let range = max - min;
        for (x, y) in res.iter().cloned().enumerate() {
            let y = (y - min) / range;
            let (y, y2) = if let Some(next) = res.get(x + 1) {
                let next = (next - min) / range;
                (y.min(next), y.max(next))
            } else {
                (y, y)
            };
            for y in (y * CURVE_SAMPLES as f32 - 1.) as u32..(y2 * CURVE_SAMPLES as f32) as u32 {
                for (xo, yo) in [(-1, 0), (0, -1), (1, 0), (0, 1)] {
                    let x = x as i32 + xo;
                    let y = y as i32 + yo;
                    if x < 0 || y < 0 || x >= CURVE_SAMPLES as i32 || y >= CURVE_SAMPLES as i32 {
                        continue;
                    }
                    image.set_color_at(x as u32, y as u32, Color::linear_rgb(0., 0., 0.)).unwrap();
                }
            }
        }

        let image_handle = world.resource_mut::<Assets<Image>>().add(image);

        let d = Curve::<f32>::domain(&self);

        world.commands().entity(root).with_children(|p| {
        p.spawn((
            Node {
                width: Val::Percent(10.),
                height: Val::Percent(90.),
                ..Default::default()
            },
            BackgroundColor(Color::linear_rgb(1., 0., 0.)),
            Text::new(format!("{}", max))
        ));
        p.spawn((
            Node {
                width: Val::Percent(90.),
                height: Val::Percent(90.),
                ..Default::default()
            },
            ImageNode {
                image: image_handle,
                ..Default::default()
            }
        ));
        p.spawn((
            Node {
                width: Val::Percent(10.),
                height: Val::Percent(10.),
                ..Default::default()
            },
            BackgroundColor(Color::linear_rgb(1., 0., 1.)),
            Text::new(format!("{}/{}", min, d.start()))
        ));
        p.spawn((
            Node {
                width: Val::Percent(90.),
                height: Val::Percent(10.),
                ..Default::default()
            },
            BackgroundColor(Color::linear_rgb(0., 0., 1.)),
            Text::new(format!("{}", d.end())),
            TextLayout {
                justify: Justify::Right,
                ..Default::default()
            },
        ));
    });
    }

    fn update(&self, mut world: DeferredWorld, root: Entity) {
        let Some(parts) = world.get::<Children>(root).map(|c| c.iter().collect::<Vec<_>>()) else {
            error!("Failed to find children of curve widget");
            return;
        };
        let x_step = self.domain().length() / CURVE_SAMPLES as f32;
        let x_start = self.domain().start();
        let mut res = Vec::with_capacity(CURVE_SAMPLES as usize);
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for x in 0..CURVE_SAMPLES {
            let sx = x_start + x as f32 * x_step;
            let r = self.sample_unchecked(sx);
            res.push(r);
            min = min.min(r);
            max = max.max(r);
        }
        
        // get image
        let image = world.get::<ImageNode>(parts[1]).expect("Should have image when it spawned").image.clone();
        let mut images = world.resource_mut::<Assets<Image>>();
        let image = images.get_mut(&image).expect("Should have image when it spawned");
        // clear image
        for x in 0..CURVE_SAMPLES {
            for y in 0..CURVE_SAMPLES {
                image.set_color_at(x, y, Color::linear_rgb(1., 1., 1.)).unwrap();
            }
        }

        // update image
        let range = max - min;
        for (x, y) in res.iter().cloned().enumerate() {
            let y = (y - min) / range;
            let (y, y2) = if let Some(next) = res.get(x + 1) {
                let next = (next - min) / range;
                (y.min(next), y.max(next))
            } else {
                (y, y)
            };
            for y in (y * CURVE_SAMPLES as f32 - 1.) as u32..(y2 * CURVE_SAMPLES as f32) as u32 {
                for (xo, yo) in [(-1, 0), (0, -1), (1, 0), (0, 1)] {
                    let x = x as i32 + xo;
                    let y = y as i32 + yo;
                    if x < 0 || y < 0 || x >= CURVE_SAMPLES as i32 || y >= CURVE_SAMPLES as i32 {
                        continue;
                    }
                    image.set_color_at(x as u32, y as u32, Color::linear_rgb(0., 0., 0.)).unwrap();
                }
            }
        }

        world.get_mut::<Text>(parts[0]).map(|mut m| m.0 = format!("{}", max));
        world.get_mut::<Text>(parts[2]).map(|mut m| m.0 = format!("{}/{}", min, self.domain().start()));
        world.get_mut::<Text>(parts[3]).map(|mut m| m.0 = format!("{}", self.domain().end()));
    }
}