#[repr(C)]
#[derive(Clone, Copy,Debug,bytemuck::Pod,bytemuck::Zeroable)]
pub struct StarInstance {
    pub position: [f32;2],
    pub scale: f32,
    pub initial_rotation: f32,
    pub speed: [f32;2],
    pub rotation_speed: f32
}

impl StarInstance {
    /* 星型インスタンスデータの定義 */
    pub const INSTANCE_VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 2,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32,
                offset: 8,
                shader_location: 3,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32,
                offset: 12,
                shader_location: 4,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 16,
                shader_location: 5,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32,
                offset: 24,
                shader_location: 6,
            },
        ],
        step_mode: wgpu::VertexStepMode::Instance,
    };
    pub fn new_vec(nums: u32) -> Vec<Self> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..nums)
            .map(|_| StarInstance {
                position: [rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)],
                scale: rng.gen_range(0.01..0.05),
                initial_rotation: rng.gen_range(0.0..std::f32::consts::PI),
                speed: [rng.gen_range(-0.01..0.01), rng.gen_range(-0.01..0.01)],
                rotation_speed: rng.gen_range(-0.01..0.01),
            })
            .collect()
    }
    pub fn get_buffer(device: &wgpu::Device,instances: &Vec<Self>) -> wgpu::Buffer {
        use wgpu::util::DeviceExt;
        return device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&instances)
            }
        );
    }
}