extern crate gfx_device_gl;
extern crate glutin;
extern crate servo as libservo;

use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::ops::{Deref, DerefMut};
use self::libservo::compositing::compositor_thread::EventLoopWaker;
use self::libservo::{gl, BrowserId};
use self::libservo::compositing::windowing::{AnimationState, WindowMethods};
use self::glutin::GlWindow;
use self::gfx_device_gl::NewTexture;
use self::libservo::euclid::{Point2D, Size2D, TypedPoint2D, TypedScale, TypedSize2D};
use self::libservo::webrender_api::{DeviceUintRect, DeviceUintSize};
use self::libservo::servo_geometry::DeviceIndependentPixel;
use self::libservo::style_traits::cursor::CursorKind;
use self::libservo::style_traits::DevicePixel;
use self::libservo::script_traits::LoadData;
use self::libservo::net_traits::net_error_list::NetError;
use self::libservo::servo_url::ServoUrl;
use self::libservo::msg::constellation_msg::{self, Key};
use self::libservo::ipc_channel::ipc::IpcSender;
use amethyst::winit::EventsLoopProxy;
use amethyst::renderer::Texture;

pub struct ServoWindow {
    pub waker: EventsLoopProxy,
    pub gl: Rc<gl::Gl>,
    pub window: Arc<GlWindow>,
    // Needs interior mutability, so that resize event can mutate it
    pub dimensions: Arc<Mutex<(u32, u32)>>,
    pub target_texture: Arc<Mutex<Option<u32>>>,
    pub frame_buffer: Arc<Mutex<Option<u32>>>,
}

impl ServoWindow where {
    pub fn get_dimensions(&self) -> (u32, u32) {
        match self.dimensions.lock() {
            Ok(d) => {
                let d = &d.clone();
                (d.0.clone(), d.1.clone())
            }
            Err(e) => {
                eprintln!("ERROR: Dimension lock for Servo implementation was poisoned, servo UI is not guaranteed to scale correctly and may cause race conditions.");
                let d = e.get_ref();
                (d.0.clone(), d.1.clone())
            }
        }
    }

    pub fn set_dimensions(&self, width: u32, height: u32) {
        match self.dimensions.lock() {
            Ok(ref mut dimensions) => {
                dimensions.0 = width;
                dimensions.1 = height;
            }
            Err(_) => {
                eprintln!("ERROR: Dimension lock for Servo implementation was poisoned, servo UI is not guaranteed to scale correctly and may cause race conditions.");
            }
        }
    }

    pub fn set_target(&self, targ: &Texture) {
        let targ = targ.raw().deref().resource();
        match self.target_texture.lock() {
            Ok(ref mut target) => {
                let mut target = target.deref_mut();
                match targ {
                    &NewTexture::Texture(t) => {
                        *target = Some(t);
                    }
                    _ => {}
                }
            }
            Err(_) => {
                eprintln!("ERROR: Target texture lock poisoned.");
            }
        }
    }

    pub fn has_target(&self) -> Result<bool, String> {
        match self.target_texture.lock() {
            Ok(target) => match *target {
                Some(_) => Ok(true),
                None => Ok(false),
            },
            Err(_) => {
                eprintln!("ERROR: Target texture lock poisoned.");
                Err("Lock poisoned".into())
            }
        }
    }
    pub fn get_target(&self) -> Option<u32> {
        match self.target_texture.lock() {
            Ok(ref target) => target.deref().clone(),
            Err(ref e) => {
                eprintln!("ERROR: Target texture lock poisoned..");
                e.get_ref().deref().clone()
            }
        }
    }

    pub fn setup_framebuffer(&self) -> Result<(), u32> {
        let (width, height) = self.get_dimensions();
        let texture = self.get_target().unwrap();
        self.gl.bind_texture(gl::TEXTURE_2D, texture.into());

        let frame_buffer = self.gl.gen_framebuffers(1)[0];
        self.gl.bind_framebuffer(gl::FRAMEBUFFER, frame_buffer);
        let depth_buffer = self.gl.gen_renderbuffers(1)[0];
        self.gl.bind_renderbuffer(gl::RENDERBUFFER, depth_buffer);
        self.gl.renderbuffer_storage(
            gl::RENDERBUFFER,
            gl::DEPTH_COMPONENT,
            width as i32,
            height as i32,
        );
        self.gl.framebuffer_renderbuffer(
            gl::FRAMEBUFFER,
            gl::DEPTH_ATTACHMENT,
            gl::RENDERBUFFER,
            depth_buffer,
        );
        self.gl.framebuffer_texture_2d(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            texture.into(),
            0,
        );
        match self.gl.check_frame_buffer_status(gl::FRAMEBUFFER) {
            gl::FRAMEBUFFER_COMPLETE => match self.frame_buffer.lock() {
                Ok(mut fb) => {
                    self.gl.bind_framebuffer(gl::FRAMEBUFFER, 0);
                    self.gl.bind_renderbuffer(gl::RENDERBUFFER, 0);
                    self.gl.bind_texture(gl::TEXTURE_2D, 0);
                    *fb = Some(frame_buffer);
                    Ok(())
                }
                Err(_) => {
                    self.gl.delete_framebuffers(&[frame_buffer]);
                    self.gl.delete_renderbuffers(&[depth_buffer]);
                    Err(0)
                }
            },
            e => {
                self.gl.delete_framebuffers(&[frame_buffer]);
                self.gl.delete_renderbuffers(&[depth_buffer]);
                Err(e)
            }
        }
    }

    /// Binds the framebuffer which has been marked using set_texture and setup_framebuffer to the
    /// render target. Will fail with Err(0) if the lock is poisoned, or the framebuffer has not
    /// been set up. Will fail with an appropriate GLenum if the framebuffer check fails. In the
    /// event of a framebuffer check failure, the framebuffer will be automatically unbound and the
    /// render target will be checked again. in the event that the unbind still fails, the error
    /// code for the framebufferless render target will be returned in leau of the original error.
    /// Otherwise the original GLEnum from binding the framebuffer shall be returned.
    pub fn enable_fb(&self) -> Result<(), ()> {
        match self.frame_buffer.lock() {
            Ok(guard) => match *guard {
                Some(fb) => {
                    self.gl.bind_framebuffer(gl::FRAMEBUFFER, fb);
                    Ok(())
                }
                None => Err(()),
            },
            Err(_) => Err(()),
        }
    }

    pub fn disable_fb(&self) {
        self.gl.bind_framebuffer(gl::FRAMEBUFFER, 0);
    }
}

struct WinitEventLoopWaker {
    waker: EventsLoopProxy,
}

impl EventLoopWaker for WinitEventLoopWaker {
    fn clone(&self) -> Box<EventLoopWaker + Send> {
        Box::new(Self {
            waker: self.waker.clone(),
        })
    }
    fn wake(&self) {
        self.waker.wakeup().unwrap();
    }
}

impl WindowMethods for ServoWindow {
    fn gl(&self) -> Rc<gl::Gl> {
        self.gl.clone()
    }

    fn framebuffer_size(&self) -> DeviceUintSize {
        let scale_factor = self.window.hidpi_factor() as u32;
        // TODO(ajeffrey): can this fail?
        let (width, height) = self.window
            .get_inner_size()
            .expect("Failed to get window inner size.");
        DeviceUintSize::new(width, height) * scale_factor
    }

    fn window_rect(&self) -> DeviceUintRect {
        let size = self.framebuffer_size();
        let origin = TypedPoint2D::zero();
        DeviceUintRect::new(origin, size)
    }

    fn size(&self) -> TypedSize2D<f32, DeviceIndependentPixel> {
        let (width, height) = self.window
            .get_inner_size()
            .expect("Failed to get window inner size.");
        TypedSize2D::new(width as f32, height as f32)
    }

    fn client_window(&self, _: BrowserId) -> (Size2D<u32>, Point2D<i32>) {
        // TODO(ajeffrey): can this fail?
        let (width, height) = self.window
            .get_outer_size()
            .expect("Failed to get window outer size.");
        let size = Size2D::new(width, height);
        // TODO(ajeffrey): can this fail?
        let (x, y) = self.window
            .get_position()
            .expect("Failed to get window position.");
        let origin = Point2D::new(x as i32, y as i32);
        (size, origin)
    }

    fn screen_size(&self, _: BrowserId) -> Size2D<u32> {
        let dimensions = self.get_dimensions();
        let size = Size2D::new(dimensions.0.into(), dimensions.1.into());
        size
    }

    fn screen_avail_size(&self, _: BrowserId) -> Size2D<u32> {
        let dimensions = self.get_dimensions();
        let size = Size2D::new(dimensions.0.into(), dimensions.1.into());
        size
    }

    fn set_animation_state(&self, _state: AnimationState) {}

    fn set_inner_size(&self, _: BrowserId, _size: Size2D<u32>) {}

    fn set_position(&self, _: BrowserId, _point: Point2D<i32>) {}

    fn set_fullscreen_state(&self, _: BrowserId, _state: bool) {}

    fn prepare_for_composite(&self, _width: usize, _height: usize) -> bool {
        match self.enable_fb() {
            Ok(()) => {
                println!("Successfully bound framebuffer");
                true
            }
            Err(()) => {
                println!("Failed to enable framebuffer");
                false
            }
        }
    }

    fn present(&self) {
        self.disable_fb();
        println!("Unbound framebuffer");
    }

    fn create_event_loop_waker(&self) -> Box<EventLoopWaker> {
        Box::new(WinitEventLoopWaker {
            waker: self.waker.clone(),
        })
    }

    fn set_page_title(&self, _: BrowserId, _title: Option<String>) {}

    fn status(&self, _: BrowserId, _status: Option<String>) {}

    fn load_start(&self, _: BrowserId) {}

    fn load_end(&self, _: BrowserId) {}

    fn history_changed(&self, _: BrowserId, _history: Vec<LoadData>, _current: usize) {}

    fn load_error(&self, _: BrowserId, _: NetError, _: String) {}

    fn head_parsed(&self, _: BrowserId) {}

    /// Has no effect on Android.
    fn set_cursor(&self, _cursor: CursorKind) {}

    fn set_favicon(&self, _: BrowserId, _: ServoUrl) {}

    /// Helper function to handle keyboard events.
    fn handle_key(
        &self,
        _: Option<BrowserId>,
        _ch: Option<char>,
        _key: Key,
        _mods: constellation_msg::KeyModifiers,
    ) {
    }

    fn allow_navigation(&self, _: BrowserId, _: ServoUrl, _response_chan: IpcSender<bool>) {}

    fn supports_clipboard(&self) -> bool {
        true
    }

    fn hidpi_factor(&self) -> TypedScale<f32, DeviceIndependentPixel, DevicePixel> {
        TypedScale::new(self.window.hidpi_factor())
    }

    fn handle_panic(&self, _: BrowserId, _reason: String, _backtrace: Option<String>) {
        // Nothing to do here yet. The crash has already been reported on the console.
    }
}
