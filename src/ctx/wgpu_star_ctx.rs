use std::{borrow::Cow, sync::Arc};
use wgpu::{util::DeviceExt, ShaderSource};
use winit::window::Window;

use crate::{time_uniform::TimeUniform, vertex::{instance::StarInstance, position::PositionVertex}};

pub struct WgpuStarCtx<'window> {
    pub surface: wgpu::Surface<'window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub render_pipeline: wgpu::RenderPipeline,
    #[allow(unused)]
    pub adapter: wgpu::Adapter
}



impl<'window> WgpuStarCtx<'window> {
    const NUM_STARS: u32 = 1000;

    /// 非同期処理での初期化関数を同期関数として扱えるようにしたもの
    /// これを通常使用する
    pub fn new(window: Arc<Window>) -> Self {
        pollster::block_on(WgpuStarCtx::new_async(window))
    }

    /// 非同期関数としての初期化関数
    /// 実際には同期関数として初期化する関数を別で実行し、
    /// その関数内で非同期を解決する
    async fn new_async(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(
            Arc::clone(&window)
        ).unwrap();
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptionsBase { 
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface)
            }
        ).await.unwrap();
        let (device,queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                ..Default::default()
            },
            None
        ).await.unwrap();

        device.on_uncaptured_error(Box::new(|err| {
            eprintln!("Device: error: {:?}",err)
        }));

        let size = window.inner_size();
        let (width,height) = (size.width.max(1),size.width.max(1));
        let surface_config = surface
            .get_default_config(&adapter, width, height)
            .unwrap();
        surface.configure(&device, &surface_config);

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Time Uniform Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None
                        },
                        count: None
                    }
                ]
            }
        );

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Star Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[]
            }
        );

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Star Shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("../star_shader.wgsl"))),
        });

        let render_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Star Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertexMain"),
                    compilation_options: Default::default(),
                    buffers: &[
                        PositionVertex::POSITION_VERTEX_LAYOUT,StarInstance::INSTANCE_VERTEX_LAYOUT
                    ],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fragmentMain"),
                    targets: &[Some(wgpu::ColorTargetState {
                        write_mask: wgpu::ColorWrites::ALL,
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                    })],
                    compilation_options: Default::default(),
                })
            }
        );

        Self {
            surface,
            device,
            queue,
            surface_config,
            render_pipeline,
            adapter
        }
    }

    
    /// リサイズ用の関数
    pub fn resize(&mut self, new_size: (u32, u32)) {
        let (width, height) = new_size;
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }


    /// 描画用関数
    pub fn draw(&mut self) {
        
        let mut vertices: Vec<PositionVertex> = PositionVertex::new_vecs(PositionVertex::STAR_VERTEX_SIZE * 2);
        let clone_vertices = vertices.clone();
        let vertices_bytes: &[u8]  = PositionVertex::vertices_byte(&clone_vertices);
        let vertex_buffer = PositionVertex::get_buffer(&self.device, vertices_bytes);

        let uniform = TimeUniform::new();
        let uniform_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Time Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );

        let bind_group_layout = self.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Time Uniform Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
            }
        );

        let uniform_bind_group = self.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Time Uniform Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    }
                ],
            }
        );

        let surface_texture = self.surface
            .get_current_texture()
            .expect("Failed to acquire next surface texture");

        let texture_view = surface_texture
            .texture
            .create_view(
                &wgpu::TextureViewDescriptor::default()
            );

        let current_time = uniform.after_duration();

        self.queue.write_buffer(
            &uniform_buffer,
            0,
            bytemuck::cast_slice(&[current_time])
        );

        let mut command_encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder")
            }
        );


        /* let mut indices: Vec<u16> = Vec::new();
        for i in 0..PositionVertex::STAR_VERTEX_SIZE * 2 {
            indices.push(i as u16);
            indices.push((i + 5) as u16);
            indices.push((i + 1) as u16 % (5 as u16 * 2));
            indices.push(5 as u16 * 2 as u16);
        }
        vertices.push(PositionVertex::CENTER);
        let indices_bytes: &[u8] = bytemuck::cast_slice(
            indices.as_slice()
        );


    
        

        /* インデックスデータをバッファとして用意 */
        let index_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                contents: indices_bytes
            }
        );

        let (uniform_buffer,uniform_bind_group) = TimeUniform::get_time_uniform_buffer_and_bindgroup(
            &self.device,
            &self.render_pipeline
        );

        

        
         */
        

        
        

        {
            let mut rpass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(
                &self.render_pipeline
            );

            rpass.set_bind_group(
                0,
                &uniform_bind_group,
                &[]
            );
            rpass.set_vertex_buffer(0, vertex_buffer.slice(..));

            let instances: Vec<StarInstance> = StarInstance::new_vec(Self::NUM_STARS);
            // インスタンスデータのバッファ化
            let instance_buffer = StarInstance::get_buffer(&self.device, &instances);
            
            rpass.set_vertex_buffer(1, instance_buffer.slice(..));
            rpass.draw(0..vertices.len() as u32,0..instances.len() as u32)

        }
        self.queue.submit(Some(command_encoder.finish()));
        surface_texture.present();
    }

}