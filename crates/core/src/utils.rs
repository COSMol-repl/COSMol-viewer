use glam::Mat4;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy)]
pub struct VisualStyle {
    pub color: Option<[f32; 3]>,
    pub opacity: f32, // 强制 0~1，默认为 1.0
    pub wireframe: bool,
    pub visible: bool,
    pub line_width: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy)]
pub struct Interaction {
    pub clickable: bool,
    pub hoverable: bool,
    pub context_menu_enabled: bool,
    // 可扩展为事件 enum，如 Click(EventCallback)
}

// -------------------- 图元结构体 --------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Shape {
    Sphere(Sphere),
    // Cube(Cube),
    // Custom(CustomShape),
    // ...
}

pub trait ToMesh {
    fn to_mesh(&self) -> MeshData;
}

impl ToMesh for Shape {
    fn to_mesh(&self) -> MeshData {
        match self {
            Shape::Sphere(s) => s.to_mesh(),
            // Shape::Cube(c) => c.to_mesh(),
            // ...
        }
    }
}

pub struct MeshData {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub colors: Option<Vec<[f32; 4]>>,
    pub transform: Option<Mat4>, // 可选位移旋转缩放
    pub is_wireframe: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Sphere {
    pub center: [f32; 3],
    pub radius: f32,
    pub quality: u32,

    pub style: VisualStyle,
    pub interaction: Interaction,
}

pub trait VisualShape {
    fn style_mut(&mut self) -> &mut VisualStyle;

    fn with_color(mut self, color: [f32; 3]) -> Self
    where
        Self: Sized,
    {
        self.style_mut().color = Some(color);
        self
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

    pub fn set_center(&mut self, center: [f32; 3]) {
        self.center = center;
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.style.color = Some(color[0..3].try_into().unwrap());
        self.style.opacity = color[3];
    }

    pub fn clickable(mut self, val: bool) -> Self {
        self.interaction.clickable = val;
        self
    }

    pub fn to_mesh(&self) -> MeshData {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();
        let mut colors = Vec::new();

        let lat_segments = 10 * self.quality;
        let lon_segments = 20 * self.quality;

        let r = self.radius;
        let [cx, cy, cz] = self.center;

        // 基础颜色（带透明度）
        let base_color = self.style.color.unwrap_or([1.0, 1.0, 1.0]);
        let alpha = self.style.opacity.clamp(0.0, 1.0);
        let color_rgba = [base_color[0], base_color[1], base_color[2], alpha];

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

                let x = cx + r * nx;
                let y = cy + r * ny;
                let z = cz + r * nz;

                vertices.push([x, y, z]);
                normals.push([nx, ny, nz]);
                colors.push(color_rgba); // 每个顶点同样颜色
            }
        }

        for i in 0..lat_segments {
            for j in 0..lon_segments {
                let first = i * (lon_segments + 1) + j;
                let second = first + lon_segments + 1;

                indices.push(first);
                indices.push(second);
                indices.push(first + 1);

                indices.push(second);
                indices.push(second + 1);
                indices.push(first + 1);
            }
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

impl VisualShape for Sphere {
    fn style_mut(&mut self) -> &mut VisualStyle {
        &mut self.style
    }
}