use std::{collections::HashMap, sync::Mutex};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use glam::{Vec3, Vec4};

use crate::{
    Shape,
    scene::{Scene, StickInstance},
    shapes::sphere::MeshTemplate,
    utils::{Interaction, Interpolatable, Logger, MeshData, VisualShape, VisualStyle},
};

static STICK_TEMPLATE_CACHE: Lazy<Mutex<HashMap<u32, MeshTemplate>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Stick {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub thickness_radius: f32,
    pub quality: u32,

    pub style: VisualStyle,
    interaction: Interaction,
}

impl Interpolatable for Stick {
    fn interpolate(&self, other: &Self, t: f32, _logger: impl Logger) -> Self {
        Self {
            start: [
                self.start[0] * (1.0 - t) + other.start[0] * t,
                self.start[1] * (1.0 - t) + other.start[1] * t,
                self.start[2] * (1.0 - t) + other.start[2] * t,
            ],
            end: [
                self.end[0] * (1.0 - t) + other.end[0] * t,
                self.end[1] * (1.0 - t) + other.end[1] * t,
                self.end[2] * (1.0 - t) + other.end[2] * t,
            ],
            thickness_radius: self.thickness_radius * (1.0 - t) + other.thickness_radius * t,
            quality: ((self.quality as f32) * (1.0 - t) + (other.quality as f32) * t) as u32,
            style: self.style.clone(), // 直接 clone，或者实现 style 插值
            interaction: self.interaction.clone(),
        }
    }
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
            thickness_radius: radius,
            quality: 6,
            style: VisualStyle {
                opacity: 1.0,
                visible: true,
                ..Default::default()
            },
            interaction: Default::default(),
        }
    }

    pub fn set_thickness(mut self, thickness: f32) -> Self {
        self.thickness_radius = thickness;
        self
    }

    pub fn set_start(mut self, start: [f32; 3]) -> Self {
        self.start = start;
        self
    }

    pub fn set_end(mut self, end: [f32; 3]) -> Self {
        self.end = end;
        self
    }

    // fn clickable(mut self, val: bool) -> Self {
    //     self.interaction.clickable = val;
    //     self
    // }

    pub fn to_mesh(&self, scale: f32) -> MeshData {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();
        let mut colors = Vec::new();

        let segments = 20 * self.quality;
        let r = self.thickness_radius;

        let start = glam::Vec3::from_array(self.start);
        let end = glam::Vec3::from_array(self.end);
        let axis = end - start;
        let height = axis.length();

        let base_color = self.style.color.unwrap_or([1.0, 1.0, 1.0]);
        let alpha = self.style.opacity.clamp(0.0, 1.0);
        let color_rgba = Vec4::new(base_color[0], base_color[1], base_color[2], alpha);

        // 构建单位 Z 轴方向的圆柱体
        for i in 0..=segments {
            let theta = (i as f32) / (segments as f32) * std::f32::consts::TAU;
            let (cos, sin) = (theta.cos(), theta.sin());
            let x = cos * r;
            let y = sin * r;

            vertices.push(Vec3::new(x, y, 0.0));
            normals.push(Vec3::new(cos, sin, 0.0));
            colors.push(color_rgba);

            vertices.push(Vec3::new(x, y, height));
            normals.push(Vec3::new(cos, sin, 0.0));
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
            let p = *v;
            let rotated = rotation * p + start;
            *v = rotated * scale;
        }

        for n in &mut normals {
            let p = *n;
            let rotated = rotation * p;
            *n = rotated * scale;
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

    pub fn get_or_generate_cylinder_mesh_template(quality: u32) -> MeshTemplate {
        let mut cache = STICK_TEMPLATE_CACHE.lock().unwrap();
        if let Some(template) = cache.get(&quality) {
            return template.clone();
        }

        let stick = Stick::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0).set_thickness(1.0);

        let mesh = stick.to_mesh(1.0);

        let template = MeshTemplate {
            vertices: mesh.vertices,
            normals: mesh.normals,
            indices: mesh.indices,
        };

        cache.insert(quality, template.clone());
        template
    }

    pub fn to_instance(&self, scale: f32) -> StickInstance {
        let base_color = self.style.color.unwrap_or([1.0, 1.0, 1.0]);
        let alpha = self.style.opacity.clamp(0.0, 1.0);
        let color = [base_color[0], base_color[1], base_color[2], alpha];

        StickInstance {
            start: self.start.map(|x| x * scale),
            end: self.end.map(|x| x * scale),
            radius: self.thickness_radius * scale,
            color,
        }
    }
}

impl VisualShape for Stick {
    fn style_mut(&mut self) -> &mut VisualStyle {
        &mut self.style
    }
}

pub trait _UpdateStick {
    fn update_stick(&mut self, id: &str, f: impl FnOnce(&mut Stick));
}

impl _UpdateStick for Scene {
    fn update_stick(&mut self, id: &str, f: impl FnOnce(&mut Stick)) {
        if let Some(Shape::Stick(stick)) = self.named_shapes.get_mut(id) {
            f(stick);
        } else {
            panic!("Stick with ID '{}' not found or is not a Stick", id);
        }
    }
}
