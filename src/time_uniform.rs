use std::time::SystemTime;

pub struct TimeUniform {
    pub time: SystemTime
}

impl TimeUniform {
    pub fn new() -> Self {
        Self {
            time: SystemTime::now()
        }
    }

    pub fn after_duration(self) -> u8 {
        std::time::SystemTime::now()
            .duration_since(self.time)
            .expect("Duration Since Error")
            .as_secs() as u8
    }

    pub fn get_time_uniform_buffer_and_bindgroup(device: &wgpu::Device,pipeline: &wgpu::RenderPipeline) -> (wgpu::Buffer,wgpu::BindGroup) {
        let uniform_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Time Uniform"),
                size: std::mem::size_of::<f32>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false
            }
        );
        let uniform_bindgroup = device.create_bind_group(
            &wgpu::BindGroupDescriptor { 
                label: Some("Uniform BindGroup"), 
                layout: &pipeline.get_bind_group_layout(0), 
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            uniform_buffer.as_entire_buffer_binding()
                        ),
                    }
                ],
            }
        );
        return (
            uniform_buffer,
            uniform_bindgroup
        )
    }
}

