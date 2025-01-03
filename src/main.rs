mod ctx;

use std::sync::Arc;

use ctx::ctx_traits::WgpuCtxBase;
use ctx::wgpu_ctx::WgpuCtx;
use ctx::wgpu_star_ctx::{self, WgpuStarCtx};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Default)]
pub struct App<'window> {
    window: Option<Arc<Window>>,
    wgpu_ctx: Option<WgpuCtx<'window>>,
    wgpu_star_ctx: Option<WgpuStarCtx<'window>>,
}


impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("wgpu winit example");
            // use Arc.
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("create window err."),
            );
            self.window = Some(window.clone());
            self.wgpu_ctx = Some(WgpuCtx::new(window.clone()));
            self.wgpu_star_ctx = Some(WgpuStarCtx::new(window.clone()));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                // macOS err: https://github.com/rust-windowing/winit/issues/3668
                // This will be fixed as winit 0.30.1.
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                /* if let (Some(wgpu_ctx), Some(window)) =
                    (self.wgpu_ctx.as_mut(), self.window.as_ref())
                {
                    wgpu_ctx.resize((new_size.width, new_size.height));
                    window.request_redraw();
                } */

               if let (Some(wgpu_star_ctx),Some(window)) = (self.wgpu_star_ctx.as_mut(),self.window.as_ref()) {
                   wgpu_star_ctx.resize((new_size.width, new_size.height));
                   window.request_redraw();
               }
            }
            WindowEvent::RedrawRequested => {
                /* if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                    wgpu_ctx.draw();
                } */
               if let Some(wgpu_star_ctx) = self.wgpu_star_ctx.as_mut() {
                   wgpu_star_ctx.draw();
               }
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app).expect("Event Loop Error");
}