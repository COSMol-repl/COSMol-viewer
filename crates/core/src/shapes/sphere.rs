use crate::Arc;
use crate::{
    Shape,
    utils::{Interaction, Interpolatable, Logger, MeshData, VisualShape, VisualStyle},
};
use dashmap::DashMap;
use glam::Vec3;
use serde::{Deserialize, Serialize};

use once_cell::sync::Lazy;

#[derive(Clone)]
pub struct MeshTemplate {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub indices: Vec<u32>,
}

static SPHERE_TEMPLATE_CACHE: Lazy<DashMap<u32, Arc<MeshTemplate>>> = Lazy::new(|| DashMap::new());

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Sphere {
    pub center: [f32; 3],
    pub radius: f32,
    pub quality: u32,

    pub style: VisualStyle,
    pub interaction: Interaction,
}

impl Interpolatable for Sphere {
    fn interpolate(&self, other: &Self, t: f32, _logger: impl Logger) -> Self {
        Self {
            center: [
                self.center[0] * (1.0 - t) + other.center[0] * t,
                self.center[1] * (1.0 - t) + other.center[1] * t,
                self.center[2] * (1.0 - t) + other.center[2] * t,
            ],
            radius: self.radius * (1.0 - t) + other.radius * t,
            quality: ((self.quality as f32) * (1.0 - t) + (other.quality as f32) * t) as u32,
            style: self.style.clone(), // 简单处理，或者给 VisualStyle 也实现 interpolate
            interaction: self.interaction.clone(), // 同上
        }
    }
}

impl Into<Shape> for Sphere {
    fn into(self) -> Shape {
        Shape::Sphere(self)
    }
}

impl Sphere {
    pub fn new(center: [f32; 3], radius: f32) -> Self {
        Self {
            center,
            radius,
            quality: 2,
            style: VisualStyle {
                opacity: 1.0,
                visible: true,
                ..Default::default()
            },
            interaction: Default::default(),
        }
    }

    pub fn set_center(mut self, center: [f32; 3]) -> Self {
        self.center = center;
        self
    }

    pub fn set_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn to_mesh(&self, _scale: f32) -> MeshData {
        return MeshData::default();

        // let template = Self::get_or_generate_sphere_mesh_template(self.quality);

        // let [cx, cy, cz] = self.center;
        // let r = self.radius;

        // let transformed_vertices: Vec<[f32; 3]> = template
        //     .vertices
        //     .iter()
        //     .map(|v| {
        //         [
        //             (v[0] * r + cx) * scale,
        //             (v[1] * r + cy) * scale,
        //             (v[2] * r + cz) * scale,
        //         ]
        //     })
        //     .collect();

        // let transformed_normals: Vec<[f32; 3]> = template
        //     .normals
        //     .iter()
        //     .map(|n| n.map(|x| x * scale)) // 你可以不乘 scale，如果只用于方向
        //     .collect();

        // let base_color = self.style.color.unwrap_or([1.0, 1.0, 1.0]);
        // let alpha = self.style.opacity.clamp(0.0, 1.0);
        // let color = [base_color[0], base_color[1], base_color[2], alpha];

        // let colors = vec![color; transformed_vertices.len()];

        // MeshData {
        //     vertices: transformed_vertices,
        //     normals: transformed_normals,
        //     indices: template.indices.clone(),
        //     colors: Some(colors),
        //     transform: None,
        //     is_wireframe: self.style.wireframe,
        // }
    }

    pub fn get_or_generate_sphere_mesh_template(quality: u32) -> Arc<MeshTemplate> {
        if let Some(entry) = SPHERE_TEMPLATE_CACHE.get(&quality) {
            return Arc::clone(entry.value());
        }

        let lat_segments = 10 * quality;
        let lon_segments = 20 * quality;

        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();

        for i in 0..=lat_segments {
            let theta = std::f32::consts::PI * (i as f32) / (lat_segments as f32);
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for j in 0..=lon_segments {
                let phi = 2.0 * std::f32::consts::PI * (j as f32) / (lon_segments as f32);
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let nx = sin_theta * cos_phi;
                let ny = cos_theta;
                let nz = sin_theta * sin_phi;

                vertices.push(Vec3::new(nx, ny, nz)); // 单位球
                normals.push(Vec3::new(nx, ny, nz));
            }
        }

        for i in 0..lat_segments {
            for j in 0..lon_segments {
                let first = i * (lon_segments + 1) + j;
                let second = first + lon_segments + 1;

                indices.push(first);
                indices.push(first + 1);
                indices.push(second);

                indices.push(second);
                indices.push(first + 1);
                indices.push(second + 1);
            }
        }

        let template = Arc::new(MeshTemplate {
            vertices,
            normals,
            indices,
        });

        // 3. 插入（如果并发下已经被别人插入，DashMap 会覆盖 or 返回旧值）
        SPHERE_TEMPLATE_CACHE.insert(quality, Arc::clone(&template));

        template
    }

    pub fn to_instance(&self, scale: f32) -> SphereInstance {
        let base_color = self.style.color.unwrap_or([1.0, 1.0, 1.0].into());
        let alpha = self.style.opacity.clamp(0.0, 1.0);
        let color = [base_color[0], base_color[1], base_color[2], alpha];

        SphereInstance::new(self.center.map(|x| x * scale), self.radius * scale, color)
    }
}

impl VisualShape for Sphere {
    fn style_mut(&mut self) -> &mut VisualStyle {
        &mut self.style
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct SphereInstance {
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 4],
}

impl SphereInstance {
    pub fn new(position: [f32; 3], radius: f32, color: [f32; 4]) -> Self {
        Self {
            position,
            radius,
            color,
        }
    }
}
