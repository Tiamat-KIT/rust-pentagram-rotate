use std::{borrow::Cow, f32::consts::PI, ops::Range, sync::Arc};

use rand::Rng;
use wgpu::{util::DeviceExt, ShaderSource};
use winit::window::Window;

use crate::ctx::ctx_traits::{WgpuCtxBase};

pub struct WgpuStarCtx<'window> {
    pub surface: wgpu::Surface<'window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub render_pipeline: wgpu::RenderPipeline,
    #[allow(unused)]
    pub adapter: wgpu::Adapter,
}

#[repr(C)]
#[derive(Clone, Copy,Debug,bytemuck::Pod,bytemuck::Zeroable)]
pub struct StarInstance {
    position: [f32;2],
    scale: f32,
    initial_rotation: f32,
    speed: [f32;2],
    rotation_speed: f32
}

impl<'window> WgpuCtxBase for WgpuStarCtx<'window> {

    /// 非同期処理での初期化関数を同期関数として扱えるようにしたもの
    /// これを通常使用する
    fn new(window: Arc<Window>) -> Self {
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

        let render_pipeline = Self::create_pipeline(
            &device,
            &surface_config.format
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
    fn resize(&mut self, new_size: (u32, u32)) {
        let (width, height) = new_size;
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }

    /// 描画用関数
    fn draw(&mut self) {
        const CENTER: [f32;2] = [0.0,0.0];
        let mut vertices: Vec<[f32;2]> = Vec::new();
        for i in 0..5  * 2 {
            let radius = if i % 2 == 0 {
                1.0
            } else {
                0.38
            };

            let angle = i as f32 * PI / 5 as f32;
            vertices.push([
                CENTER[0] + angle.cos() * radius,
                CENTER[1] + angle.sin() * radius
            ]);
        }

        let mut indices: Vec<u16> = Vec::new();
        for i in 0..5 * 2 {
            indices.push(i as u16);
            indices.push((i + 5) as u16);
            indices.push((i + 1) as u16 % (5 as u16 * 2));
            indices.push(5 as u16 * 2 as u16);
        }
        vertices.push(CENTER);


        /* 頂点データをバッファとして用意 */
        let vertex_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&vertices)
            }
        );
        self.queue.write_buffer(
            &vertex_buffer,
            0,
            bytemuck::cast_slice(&vertices)
        );

        /* インデックスデータをバッファとして用意 */
        let index_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&indices)
            }
        );

        self.queue.write_buffer(
            &index_buffer,
            0,
            bytemuck::cast_slice(&indices)
        );
        
        // 乱数を生成するオブジェクトの生成
        let mut rng = rand::thread_rng();

        // 星型インスタンスの大量生成
        let star_insntances: Vec<StarInstance> = (0..1000)
            .map(|_| StarInstance {
                position: [rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)],
                scale: rng.gen_range(0.01..0.05),
                initial_rotation: rng.gen_range(0.0..std::f32::consts::PI),
                speed: [rng.gen_range(-0.01..0.01), rng.gen_range(-0.01..0.01)],
                rotation_speed: rng.gen_range(-0.01..0.01),
            })
            .collect();
        
        // インスタンスデータのバッファ化
        let instance_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&star_insntances)
            }
        );

        self.queue.write_buffer(
            &instance_buffer,
            0,
            bytemuck::cast_slice(&star_insntances)
        );

        // 時間経過を伝えるデータをUniform Bufferとして作成
        let uniform_buffer = self.device.create_buffer(
            &wgpu::BufferDescriptor {
                label: None,
                size: std::mem::size_of::<[f32; 1]>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );

        let uniform_bind_group = self.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: None,
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
                ),
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

        let star_pipeline = Self::create_pipeline(&self.device, &self.surface_config.format);

        
        // 実行直前の時間を取得
        let start_time  = std::time::SystemTime::now();


        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            let after_time = std::time::SystemTime::now()
                .duration_since(start_time)
                .expect("Caculate Error");

            self.queue.write_buffer(
                &uniform_buffer,
                0,
                &[after_time.as_secs() as u8]
            );

            /*
             * 1. Pipeline設定
             * 2. 0番目にuniformのBindGroupを設定
             * 3. 頂点バッファを0番目,インスタンスバッファを1番目にセット
             * 4. indexバッファをuint16の型として登録
             * 5. indexバッファを使った描画を行う
             * 6. 設定を終了
             *
             * TypeScriptだとこう
             *  renderPass.setPipeline(pipeline);
                renderPass.setBindGroup(0, uniformBindGroup);
                renderPass.setVertexBuffer(0, vertexBuffer);
                renderPass.setVertexBuffer(1, instanceBuffer);
                renderPass.setIndexBuffer(indexBuffer, 'uint16');
                renderPass.drawIndexed(starData.indices.length, NUM_STARS, 0, 0, 0);
                renderPass.end();
             */

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
                indices.len(),
                5,
                instances
            );
            

            /* rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..3, 0..1); */
        }
        self.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }

    /// RenderPipeline作成関数
    fn create_pipeline(
        device: &wgpu::Device,
        format: &wgpu::TextureFormat
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Star Shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("../star_shader.wgsl"))),
        });

        return device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Star Pipeline"),
                layout: None,
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertexMain"),
                    compilation_options: Default::default(),
                    buffers: &[
                        /* 頂点座標用データの定義 */ 
                        wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            }
                        ],
                        step_mode: wgpu::VertexStepMode::Vertex
                    },
                    /* 星型インスタンスデータの定義 */
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<StarInstance>() as wgpu::BufferAddress,
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
                    }],
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
                        format: format.clone(),
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
    }
}