use std::time::{Duration, SystemTime};

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy,Debug,Pod,Zeroable)]
pub struct TimeUniform {
    pub time: f32,
    _padding: [u8;12]
}

impl TimeUniform {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            _padding: [0;12]
        }
    }

    pub fn after_duration(&self) -> Self {
        Self {
            time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f32(),
            _padding: [0; 12],
        }
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

