use winit::window::Window;
use std::sync::Arc;

pub struct State<'a> {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'a>,
}

impl<'a> State<'a> {
    pub async fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        Self {
            instance,
            surface,
        }
    }
}