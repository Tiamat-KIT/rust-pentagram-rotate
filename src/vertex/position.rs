#[repr(C)]
#[derive(Clone, Copy,Debug,bytemuck::Pod,bytemuck::Zeroable)]
pub struct PositionVertex {
    pub position: [f32;2]
}

impl PositionVertex {
    pub const STAR_VERTEX_SIZE: u32 = 5;
    /* 中心データ */
    pub const CENTER: Self = Self {
        position: [0.0,0.0]
    };

    /* 頂点座標用データの定義 */ 
    pub const POSITION_VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }
        ],
        step_mode: wgpu::VertexStepMode::Vertex
    };

    pub fn new_vecs(nums: u32) -> Vec<Self> {
        use std::f32::consts::PI;
        return (0..nums)
            .map(|f| {
                let radius = if f % 2 == 0 {
                    1.0
                } else {
                    0.38
                };
        
                let angle = f as f32 * PI / 5 as f32;
                Self {
                    position: [
                        PositionVertex::CENTER.position[0] + angle.cos() * radius,
                        PositionVertex::CENTER.position[1] + angle.sin() * radius
                    ]
                }
            })
            .collect()
    }

    pub fn vertices_byte(vertices: &Vec<Self>) -> &[u8] {
        use bytemuck;
        return bytemuck::cast_slice(
            vertices.as_slice()
        );
    }

    pub fn get_buffer(device: &wgpu::Device,vertices_bytes: &[u8]) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                contents: vertices_bytes
            }
        )
    }
}