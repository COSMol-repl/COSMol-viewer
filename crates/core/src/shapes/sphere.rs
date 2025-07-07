use serde::{Deserialize, Serialize};

use crate::{scene::Scene, utils::{Interaction, MeshData, VisualShape, VisualStyle}, Shape};

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone)]
struct SphereTemplate {
    vertices: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
}

static SPHERE_TEMPLATE_CACHE: Lazy<Mutex<HashMap<u32, SphereTemplate>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

fn get_or_generate_template(quality: u32) -> SphereTemplate {
    let mut cache = SPHERE_TEMPLATE_CACHE.lock().unwrap();

    if let Some(template) = cache.get(&quality) {
        return template.clone(); // 直接返回已有的
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

            vertices.push([nx, ny, nz]); // 单位球
            normals.push([nx, ny, nz]);
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

    let template = SphereTemplate {
        vertices,
        normals,
        indices,
    };

    cache.insert(quality, template.clone());

    template
}


#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Sphere {
    pub center: [f32; 3],
    pub radius: f32,
    pub quality: u32,

    pub style: VisualStyle,
    pub interaction: Interaction,
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

    pub fn center(mut self, center: [f32; 3])-> Self {
        self.center = center;
        self
    }

    pub fn set_radius(mut self, radius: f32)-> Self {
        self.radius = radius;
        self
    }

    pub fn clickable(mut self, val: bool) -> Self {
        self.interaction.clickable = val;
        self
    }

    // pub fn to_mesh(&self, scale: f32) -> MeshData {
    //     let mut vertices = Vec::new();
    //     let mut normals = Vec::new();
    //     let mut indices = Vec::new();
    //     let mut colors = Vec::new();

    //     let lat_segments = 10 * self.quality;
    //     let lon_segments = 20 * self.quality;

    //     let r = self.radius;
    //     let [cx, cy, cz] = self.center;

    //     // 基础颜色（带透明度）
    //     let base_color = self.style.color.unwrap_or([1.0, 1.0, 1.0]);
    //     let alpha = self.style.opacity.clamp(0.0, 1.0);
    //     let color_rgba = [base_color[0], base_color[1], base_color[2], alpha];

    //     for i in 0..=lat_segments {
    //         let theta = std::f32::consts::PI * (i as f32) / (lat_segments as f32);
    //         let sin_theta = theta.sin();
    //         let cos_theta = theta.cos();

    //         for j in 0..=lon_segments {
    //             let phi = 2.0 * std::f32::consts::PI * (j as f32) / (lon_segments as f32);
    //             let sin_phi = phi.sin();
    //             let cos_phi = phi.cos();

    //             let nx = sin_theta * cos_phi;
    //             let ny = cos_theta;
    //             let nz = sin_theta * sin_phi;

    //             let x = cx + r * nx;
    //             let y = cy + r * ny;
    //             let z = cz + r * nz;

    //             vertices.push([x, y, z].map(|x| x * scale));
    //             normals.push([nx, ny, nz].map(|x| x * scale));
    //             colors.push(color_rgba); // 每个顶点同样颜色
    //         }
    //     }

    //     for i in 0..lat_segments {
    //         for j in 0..lon_segments {
    //             let first = i * (lon_segments + 1) + j;
    //             let second = first + lon_segments + 1;

    //             indices.push(first);
    //             indices.push(first + 1);
    //             indices.push(second);

    //             indices.push(second);
    //             indices.push(first + 1);
    //             indices.push(second + 1);
    //         }
    //     }

    //     MeshData {
    //         vertices,
    //         normals,
    //         indices,
    //         colors: Some(colors),
    //         transform: None,
    //         is_wireframe: self.style.wireframe,
    //     }
    // }
        pub fn to_mesh(&self, scale: f32) -> MeshData {
        let template = get_or_generate_template(self.quality);

        let [cx, cy, cz] = self.center;
        let r = self.radius;

        let transformed_vertices: Vec<[f32; 3]> = template.vertices.iter()
            .map(|v| [
                (v[0] * r + cx) * scale,
                (v[1] * r + cy) * scale,
                (v[2] * r + cz) * scale,
            ])
            .collect();

        let transformed_normals: Vec<[f32; 3]> = template.normals
            .iter()
            .map(|n| n.map(|x| x * scale)) // 你可以不乘 scale，如果只用于方向
            .collect();

        let base_color = self.style.color.unwrap_or([1.0, 1.0, 1.0]);
        let alpha = self.style.opacity.clamp(0.0, 1.0);
        let color = [base_color[0], base_color[1], base_color[2], alpha];

        let colors = vec![color; transformed_vertices.len()];

        MeshData {
            vertices: transformed_vertices,
            normals: transformed_normals,
            indices: template.indices.clone(),
            colors: Some(colors),
            transform: None,
            is_wireframe: self.style.wireframe,
        }

    }
}

impl VisualShape for Sphere {
    fn style_mut(&mut self) -> &mut VisualStyle {
        &mut self.style
    }
}

pub trait UpdateSphere {
    fn update_sphere(&mut self, id: &str, f: impl FnOnce(&mut Sphere));
}

impl UpdateSphere for Scene {
    fn update_sphere(&mut self, id: &str, f: impl FnOnce(&mut Sphere)) {
        if let Some(Shape::Sphere(sphere)) = self.named_shapes.get_mut(id) {
            f(sphere);
        } else {
            panic!("Sphere with ID '{}' not found or is not a Sphere", id);
        }
    }
}