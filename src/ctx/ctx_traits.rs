use std::sync::Arc;

use winit::window::Window;

pub trait WgpuCtxBase {
    fn new(window: Arc<Window>) -> Self;
    async fn new_async(window: Arc<Window>) -> Self;
    fn resize(&mut self,new_size: (u32,u32));
    fn draw(&mut self);
    fn create_pipeline(device: &wgpu::Device,format: &wgpu::TextureFormat) -> wgpu::RenderPipeline;
}