use serde::{Deserialize, Serialize};

use crate::{
    Shape,
    scene::Scene,
    utils::{Interaction, MeshData, VisualShape, VisualStyle},
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Stick {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub radius: f32,
    pub quality: u32,

    pub style: VisualStyle,
    pub interaction: Interaction,
}

impl Into<Shape> for Stick {
    fn into(self) -> Shape {
        Shape::Stick(self)
    }
}

impl Stick {
    pub fn new(start: [f32; 3], end: [f32; 3], radius: f32) -> Self {
        Self {
            start,
            end,
            radius,
            quality: 6,
            style: VisualStyle {
                opacity: 1.0,
                visible: true,
                ..Default::default()
            },
            interaction: Default::default(),
        }
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn start(mut self, start: [f32; 3]) -> Self {
        self.start = start;
        self
    }

    pub fn end(mut self, end: [f32; 3]) -> Self {
        self.end = end;
        self
    }

    pub fn clickable(mut self, val: bool) -> Self {
        self.interaction.clickable = val;
        self
    }

    pub fn to_mesh(&self, scale: f32) -> MeshData {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();
        let mut colors = Vec::new();

        let segments = 20 * self.quality;
        let r = self.radius;

        let start = glam::Vec3::from_array(self.start);
        let end = glam::Vec3::from_array(self.end);
        let axis = end - start;
        let height = axis.length();

        let base_color = self.style.color.unwrap_or([1.0, 1.0, 1.0]);
        let alpha = self.style.opacity.clamp(0.0, 1.0);
        let color_rgba = [base_color[0], base_color[1], base_color[2], alpha];

        // 构建单位 Z 轴方向的圆柱体
        for i in 0..=segments {
            let theta = (i as f32) / (segments as f32) * std::f32::consts::TAU;
            let (cos, sin) = (theta.cos(), theta.sin());
            let x = cos * r;
            let y = sin * r;

            vertices.push([x, y, 0.0]);
            normals.push([cos, sin, 0.0]);
            colors.push(color_rgba);

            vertices.push([x, y, height]);
            normals.push([cos, sin, 0.0]);
            colors.push(color_rgba);
        }

        for i in 0..segments {
            let idx = i * 2;
            indices.push(idx + 2);
            indices.push(idx + 1);
            indices.push(idx);

            indices.push(idx + 2);
            indices.push(idx + 3);
            indices.push(idx + 1);
        }

        // 对齐旋转：Z -> axis
        let up = glam::Vec3::Z;
        let rotation = glam::Quat::from_rotation_arc(up, axis.normalize());

        for v in &mut vertices {
            let p = glam::Vec3::from_array(*v);
            let rotated = rotation * p + start;
            *v = rotated.to_array().map(|x| x * scale);
        }

        for n in &mut normals {
            let p = glam::Vec3::from_array(*n);
            let rotated = rotation * p;
            *n = rotated.to_array().map(|x| x * scale);
        }

        MeshData {
            vertices,
            normals,
            indices,
            colors: Some(colors),
            transform: None,
            is_wireframe: self.style.wireframe,
        }
    }
}

impl VisualShape for Stick {
    fn style_mut(&mut self) -> &mut VisualStyle {
        &mut self.style
    }
}

pub trait UpdateStick {
    fn update_stick(&mut self, id: &str, f: impl FnOnce(&mut Stick));
}

impl UpdateStick for Scene {
    fn update_stick(&mut self, id: &str, f: impl FnOnce(&mut Stick)) {
        if let Some(Shape::Stick(stick)) = self.named_shapes.get_mut(id) {
            f(stick);
        } else {
            panic!("Stick with ID '{}' not found or is not a Stick", id);
        }
    }
}
