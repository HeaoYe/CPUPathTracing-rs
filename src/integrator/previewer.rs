use super::{Integrator, NormalIntegrator};
use crate::{
    THREAD_POOL,
    camera::{Camera, CameraController, Direction},
    scene::Scene,
};
use std::{collections::HashSet, num::NonZeroU32, sync::Arc, time::Instant};
use winit::{
    event::{DeviceEvent, ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::CursorGrabMode,
};

#[cfg(debug_assertions)]
use super::{BoundsTestIntegrator, PrimitiveTestIntegrator};

enum RenderMode {
    Integrator,
    Normal,
    #[cfg(debug_assertions)]
    BoundsTest,
    #[cfg(debug_assertions)]
    PrimitiveTest,
}

struct Previewer<'a, 'b, I> {
    controller: CameraController<'a>,
    scene: &'a Scene<'b>,

    integrator: &'a I,
    normal_integrator: NormalIntegrator,

    #[cfg(debug_assertions)]
    bounds_test_integrator: BoundsTestIntegrator,
    #[cfg(debug_assertions)]
    primitive_test_integrator: PrimitiveTestIntegrator,

    render_mode: RenderMode,

    scale: f32,
    fps: f32,
    film_resolution: glam::USizeVec2,

    current_spp: usize,
}

impl<'a, 'b, I> Previewer<'a, 'b, I>
where
    'b: 'a,
    I: Integrator,
{
    fn new(camera: &'a mut Camera, scene: &'a Scene<'b>, integrator: &'a I) -> Self {
        let film_resolution = glam::usizevec2(camera.film.width(), camera.film.height());

        Self {
            controller: CameraController::new(camera),
            scene,
            integrator,
            normal_integrator: NormalIntegrator,

            #[cfg(debug_assertions)]
            bounds_test_integrator: BoundsTestIntegrator,
            #[cfg(debug_assertions)]
            primitive_test_integrator: PrimitiveTestIntegrator,

            render_mode: RenderMode::Integrator,
            scale: 0.2,
            fps: 30.0,
            film_resolution,
            current_spp: 0,
        }
    }

    fn set_resolution(&mut self, scale: f32) {
        let scale = scale.clamp(0.0, 1.0);
        if scale == self.scale {
            return;
        }

        self.scale = scale;
        let resolution = (scale * self.film_resolution.as_vec2())
            .round()
            .as_usizevec2()
            .max(glam::USizeVec2::ONE);
        self.controller.set_resolution(resolution.x, resolution.y);
        self.current_spp = 0;
    }

    fn adjust_resolution(&mut self, dt: f32) {
        let spp_per_frame = self.spp_per_frame();
        if self.current_spp as f32 / spp_per_frame as f32 > self.fps * 3.0 {
            return;
        }

        let expected_dt = 1.0 / self.fps;
        if (expected_dt - dt).abs() / expected_dt > 0.4 {
            let new_scale = self.scale * (1.0 + 0.1 * ((expected_dt / dt).sqrt() - 1.0));
            self.set_resolution(new_scale);
        }
    }

    fn spp_per_frame(&self) -> usize {
        match self.render_mode {
            RenderMode::Integrator => 4,
            _ => 1,
        }
    }

    fn next_render_mode(&mut self) {
        self.render_mode = match self.render_mode {
            RenderMode::Integrator => RenderMode::Normal,
            #[cfg(not(debug_assertions))]
            RenderMode::Normal => RenderMode::Integrator,

            #[cfg(debug_assertions)]
            RenderMode::Normal => RenderMode::BoundsTest,
            #[cfg(debug_assertions)]
            RenderMode::BoundsTest => RenderMode::PrimitiveTest,
            #[cfg(debug_assertions)]
            RenderMode::PrimitiveTest => RenderMode::Integrator,
        }
    }
}

impl<'a, 'b, I> Previewer<'a, 'b, I>
where
    'b: 'a,
    I: Integrator + Sync,
{
    fn render_frame(&mut self) {
        if self.current_spp == 0 {
            self.controller.camera.film.clear();
        }

        let spp = self.spp_per_frame();
        let current_spp = self.current_spp;
        let scene = self.scene;
        let camera = &self.controller.camera.model;
        let integrator: &(dyn Integrator + Sync) = match self.render_mode {
            RenderMode::Integrator => self.integrator,
            RenderMode::Normal => &self.normal_integrator,
            #[cfg(debug_assertions)]
            RenderMode::BoundsTest => &self.bounds_test_integrator,
            #[cfg(debug_assertions)]
            RenderMode::PrimitiveTest => &self.primitive_test_integrator,
        };
        THREAD_POOL.parallel_for_2d(
            self.controller.camera.film.width(),
            self.controller.camera.film.height(),
            self.controller.camera.film.as_slice_mut(),
            move |x, y, pixel| {
                for i in 1..=spp {
                    if let Some(sample) = integrator.integrate(x, y, current_spp + i, camera, scene)
                    {
                        pixel.add_sample(sample);
                    }
                }
            },
        );
        self.current_spp += spp;
    }
}

struct PreviewApp<'a, 'b, I> {
    previewer: Previewer<'a, 'b, I>,
    context: softbuffer::Context<winit::event_loop::OwnedDisplayHandle>,

    window: Option<Arc<winit::window::Window>>,
    surface: Option<
        softbuffer::Surface<winit::event_loop::OwnedDisplayHandle, Arc<winit::window::Window>>,
    >,

    pressed_keys: HashSet<KeyCode>,
    grabbed: bool,
    last_render_dt: Option<f32>,

    enter_pressed: bool,
}

impl<I> PreviewApp<'_, '_, I> {
    fn toggle_grabbed(&mut self) {
        self.grabbed = !self.grabbed;

        let Some(window) = &self.window else {
            return;
        };

        if self.grabbed {
            window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
                .unwrap();
            window.set_cursor_visible(false);
        } else {
            window.set_cursor_grab(CursorGrabMode::None).unwrap();
            window.set_cursor_visible(true);
        }
    }

    fn update_camera(&mut self, dt: f32) {
        if !self.grabbed {
            return;
        }

        let mut moved = false;
        if self.pressed_keys.contains(&KeyCode::KeyW) {
            self.previewer.controller.translate(dt, Direction::Forward);
            moved = true;
        }
        if self.pressed_keys.contains(&KeyCode::KeyS) {
            self.previewer.controller.translate(dt, Direction::Backward);
            moved = true;
        }
        if self.pressed_keys.contains(&KeyCode::KeyA) {
            self.previewer.controller.translate(dt, Direction::Left);
            moved = true;
        }
        if self.pressed_keys.contains(&KeyCode::KeyD) {
            self.previewer.controller.translate(dt, Direction::Right);
            moved = true;
        }
        if self.pressed_keys.contains(&KeyCode::Space) {
            self.previewer.controller.translate(dt, Direction::Up);
            moved = true;
        }
        if self.pressed_keys.contains(&KeyCode::ShiftLeft) {
            self.previewer.controller.translate(dt, Direction::Down);
            moved = true;
        }
        if moved {
            self.previewer.current_spp = 0;
        }
    }

    fn present(&mut self) {
        let surface = self.surface.as_mut().unwrap();
        let mut buffer = surface.buffer_mut().unwrap();
        let film = &self.previewer.controller.camera.film;
        film.write_rgb_buffer(
            buffer.width().get() as usize,
            buffer.height().get() as usize,
            &mut buffer,
        );
        buffer.present().unwrap();
    }

    fn exit(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.set_visible(false);
        }
        self.surface = None;
        self.window = None;
        event_loop.exit();
    }
}

impl<'a, 'b, I> winit::application::ApplicationHandler for PreviewApp<'a, 'b, I>
where
    'b: 'a,
    I: Integrator + Sync,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_size = winit::dpi::PhysicalSize::new(
            self.previewer.film_resolution.x as u32,
            self.previewer.film_resolution.y as u32,
        );
        let mut attributes = winit::window::Window::default_attributes()
            .with_title("CPU Path Tracing")
            .with_inner_size(window_size)
            .with_resizable(false)
            .with_decorations(false);
        if let Some(monitor) = event_loop.primary_monitor() {
            let monitor_position = monitor.position();
            let monitor_size = monitor.size();
            let position = winit::dpi::PhysicalPosition::new(
                monitor_position.x + (monitor_size.width as i32 - window_size.width as i32) / 2,
                monitor_position.y + (monitor_size.height as i32 - window_size.height as i32) / 2,
            );
            attributes = attributes.with_position(position);
        }
        let window = Arc::new(event_loop.create_window(attributes).unwrap());

        let mut surface = softbuffer::Surface::new(&self.context, window.clone()).unwrap();
        surface
            .resize(
                NonZeroU32::new(window.inner_size().width).unwrap(),
                NonZeroU32::new(window.inner_size().height).unwrap(),
            )
            .unwrap();

        self.surface = Some(surface);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.exit(event_loop);
            }
            WindowEvent::Focused(false) => {
                self.pressed_keys.clear();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let PhysicalKey::Code(key) = event.physical_key else {
                    return;
                };

                match event.state {
                    ElementState::Pressed => {
                        self.pressed_keys.insert(key);
                    }
                    ElementState::Released => {
                        self.pressed_keys.remove(&key);

                        match key {
                            KeyCode::Escape => {
                                self.exit(event_loop);
                            }
                            KeyCode::Enter => {
                                self.enter_pressed = true;
                                self.exit(event_loop);
                            }
                            KeyCode::Tab => {
                                self.previewer.next_render_mode();
                                self.previewer.current_spp = 0;
                            }
                            KeyCode::Equal => {
                                self.previewer.fps += 1.0;
                                println!("FPS: {}", self.previewer.fps);
                            }
                            KeyCode::Minus => {
                                self.previewer.fps = (self.previewer.fps - 1.0).max(1.0);
                                println!("FPS: {}", self.previewer.fps);
                            }
                            KeyCode::CapsLock => {
                                self.toggle_grabbed();
                            }
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } if self.grabbed => {
                let delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(position) => position.y as f32,
                };
                self.previewer.controller.zoom(delta);
                self.previewer.current_spp = 0;
            }
            WindowEvent::RedrawRequested => {
                if let Some(render_dt) = self.last_render_dt {
                    self.previewer.adjust_resolution(render_dt);
                    self.update_camera(render_dt);
                }

                let start = Instant::now();
                self.previewer.render_frame();
                self.last_render_dt = Some(start.elapsed().as_secs_f32().max(0.001));

                if let Some(window) = &self.window {
                    window.pre_present_notify();
                    self.present();
                }
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if !self.grabbed {
            return;
        }

        if let DeviceEvent::MouseMotion { delta } = event {
            if delta.0 == 0.0 && delta.1 == 0.0 {
                return;
            }

            self.previewer
                .controller
                .turn(glam::vec2(delta.0 as f32, delta.1 as f32));
            self.previewer.current_spp = 0;
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

pub fn preview<'a, 'b, I>(integrator: &'a I, camera: &'a mut Camera, scene: &'a Scene<'b>) -> bool
where
    'b: 'a,
    I: Integrator + Sync,
{
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let context = softbuffer::Context::new(event_loop.owned_display_handle()).unwrap();

    let mut app = PreviewApp {
        previewer: Previewer::new(camera, scene, integrator),
        context,
        window: None,
        surface: None,
        pressed_keys: HashSet::new(),
        grabbed: false,
        last_render_dt: None,
        enter_pressed: false,
    };
    event_loop.run_app(&mut app).unwrap();

    let raw_resolution = app.previewer.film_resolution;
    app.previewer
        .controller
        .set_resolution(raw_resolution.x, raw_resolution.y);
    app.previewer.controller.print();

    app.enter_pressed
}
