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

        let size = window.inner_size();
        let (width,height) = (size.width.max(1),size.width.max(1));
        let surface_config = surface.get_default_config(&adapter, width, height)
            .unwrap();
        surface.configure(&device, &surface_config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Star Shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("../star_shader.wgsl"))),
        });

        let render_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Star Pipeline"),
                layout: None,
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
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fragmentMain"),
                    targets: &[Some(wgpu::ColorTargetState {
                        write_mask: wgpu::ColorWrites::ALPHA,
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::One,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
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

        let mut indices: Vec<u16> = Vec::new();
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


        /* 頂点データをバッファとして用意 */
        let vertex_buffer = PositionVertex::get_buffer(&self.device, vertices_bytes);
        self.queue.write_buffer(
            &vertex_buffer,
            0,
            vertices_bytes
        );

        /* インデックスデータをバッファとして用意 */
        let index_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                contents: indices_bytes
            }
        );

        self.queue.write_buffer(
            &index_buffer,
            0,
            indices_bytes
        );
        let (uniform_buffer,uniform_bind_group) = TimeUniform::get_time_uniform_buffer_and_bindgroup(
            &self.device,
            &self.render_pipeline
        );

        

        let surface_texture = self.surface
            .get_current_texture()
            .expect("Fails Surface Texture Gets");

        let texture_view = surface_texture
            .texture
            .create_view(
                &wgpu::TextureViewDescriptor::default()
            );
        
        let mut command_encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder")
            }
        );

        
        // 実行直前の時間を取得
        let start_time  = TimeUniform::new();

        {
            let mut rpass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            /*
             * 時間経過の概念を、Uniform Bufferを通して、
             * 常に変化し続けるstd::timeのデータを渡す
             * 
             * ※WebAssemblyはwasm-timeで動かさないとエラーを出して成功しない
             */
            self.queue.write_buffer(
                &uniform_buffer,
                0,
                &[start_time.after_duration()]
            );

            let instances: Vec<StarInstance> = StarInstance::new_vec(Self::NUM_STARS);
            // インスタンスデータのバッファ化
            let instance_buffer = StarInstance::get_buffer(&self.device, &instances);
            
            self.queue.write_buffer(
                &instance_buffer,
                0,
                bytemuck::cast_slice(&instances)
            );

            /*
             * 1. Pipeline設定
             * 2. 0番目にuniformのBindGroupを設定
             * 3. 頂点バッファを0番目,インスタンスバッファを1番目にセット
             * 4. indexバッファをuint16の型として登録
             * 5. indexバッファを使った描画を行う
             * 6. 設定を終了 */

            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(
                0,
                &uniform_bind_group,
                &[0]
            );


            rpass.set_vertex_buffer(
                0,
                vertex_buffer.slice(..)
            );

            rpass.set_vertex_buffer(
                1,
                instance_buffer.slice(..)
            );

            rpass.set_index_buffer(
                index_buffer.slice(..),
                wgpu::IndexFormat::Uint16
            );

            rpass.draw_indexed(
                0..indices.len() as u32,
                0,
                0..instances.len() as u32
            );

        }
        self.queue.submit(Some(command_encoder.finish()));
        surface_texture.present();
    }

}