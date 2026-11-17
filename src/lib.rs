//! Button Sim running on the Rust/WolfCraft technology stack.
//! The engine is linked from the successful WolfCraft revision.

pub mod game;

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    game: game::Game,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title("Button Sim")
                .with_inner_size(winit::dpi::LogicalSize::new(420.0, 780.0));
            self.window = Some(Arc::new(event_loop.create_window(attrs).expect("window")));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::MouseInput { state: winit::event::ElementState::Pressed, .. } => self.game.tap(),
            WindowEvent::Touch(touch) if touch.phase == winit::event::TouchPhase::Started => self.game.tap(),
            _ => {}
        }
    }
}

pub fn run_desktop() {
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("application loop");
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: winit::platform::android::activity::AndroidApp) {
    use winit::event_loop::EventLoopBuilder;
    use winit::platform::android::EventLoopBuilderExtAndroid;
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));
    let event_loop = EventLoopBuilder::default().with_android_app(android_app).build().expect("android event loop");
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("application loop");
}
