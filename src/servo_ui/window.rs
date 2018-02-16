extern crate glutin;
extern crate servo as libservo;

use self::libservo::compositing::compositor_thread::EventLoopWaker;
use self::libservo::{gl, BrowserId};
use self::libservo::compositing::windowing::{AnimationState, WindowEvent, WindowMethods};
use self::glutin::{GlContext, GlWindow};
use std::sync::Arc;
use std::rc::Rc;
use self::libservo::euclid::{Point2D, Size2D, TypedPoint2D, TypedScale, TypedSize2D};
use self::libservo::webrender_api::{DeviceUintRect, DeviceUintSize};
use self::libservo::servo_geometry::DeviceIndependentPixel;
use self::libservo::style_traits::cursor::CursorKind;
use self::libservo::style_traits::DevicePixel;
use self::libservo::script_traits::LoadData;
use self::libservo::net_traits::net_error_list::NetError;
use self::libservo::servo_url::ServoUrl;
use self::libservo::msg::constellation_msg::{self, Key};
use self::libservo::ipc_channel::ipc::{self, IpcSender};
use self::libservo::Servo;
use std::ops::Drop;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use amethyst::winit::EventsLoopProxy;

struct WinitEventLoopWaker {
    proxy: Arc<EventsLoopProxy>,
}

impl EventLoopWaker for WinitEventLoopWaker {
    fn clone(&self) -> Box<EventLoopWaker + Send> {
        Box::new(Self {
            proxy: self.proxy.clone(),
        })
    }
    fn wake(&self) {
        self.proxy
            .wakeup()
            .expect("Failed to wake up winit event loop");
    }
}

pub struct ServoWindow {
    servo: Arc<RefCell<Servo<UiWindow>>>,
}

impl ServoWindow {
    pub fn new(
        waker: Arc<EventsLoopProxy>,
        win: Arc<GlWindow>,
        screen_dimensions: (f32, f32),
    ) -> Self {
        let mut servo = Servo::new(UiWindow::new(waker, win, screen_dimensions));
        let url = ServoUrl::parse("file:///home/matthew/git/openrlfb/test.html").unwrap();
        let (sender, receiver) = ipc::channel().unwrap();
        servo.handle_events(vec![WindowEvent::NewBrowser(url, sender)]);
        let id = receiver.recv().unwrap();
        servo.handle_events(vec![WindowEvent::SelectBrowser(id)]);
        Self {
            servo: Arc::new(RefCell::new(servo)),
        }
    }
}

struct UiWindow {
    window: Arc<GlWindow>,
    gl: Rc<gl::Gl>,
    screen_dimensions: Size2D<u32>,
    waker: WinitEventLoopWaker,
}

impl UiWindow {
    pub fn new(
        waker: Arc<EventsLoopProxy>,
        win: Arc<GlWindow>,
        screen_dimensions: (f32, f32),
    ) -> Rc<Self> {
        let gl = unsafe {
            win.context()
                .make_current()
                .expect("Failed to make resource GlWindow currenty");
            gl::GlFns::load_with(|s| win.context().get_proc_address(s) as *const _)
        };
        let ui_window = Rc::new(Self {
            window: win,
            gl: gl,
            screen_dimensions: Size2D::new(screen_dimensions.0 as u32, screen_dimensions.1 as u32),
            waker: WinitEventLoopWaker { proxy: waker },
        });
        ui_window
    }
}

impl WindowMethods for UiWindow {
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
        let size = Size2D::new(self.screen_dimensions.width, self.screen_dimensions.height);
        println!("Screen size: {:?}", size);
        size
    }

    fn screen_avail_size(&self, _: BrowserId) -> Size2D<u32> {
        let size = Size2D::new(self.screen_dimensions.width, self.screen_dimensions.height);
        println!("Available screen size: {:?}", size);
        size
    }

    fn set_animation_state(&self, state: AnimationState) {
        println!("Setting animation state");
    }

    fn set_inner_size(&self, _: BrowserId, size: Size2D<u32>) {}

    fn set_position(&self, _: BrowserId, point: Point2D<i32>) {
        println!("Setting position");
    }

    fn set_fullscreen_state(&self, _: BrowserId, _state: bool) {}

    fn present(&self) {
        println!("Presenting");
    }

    fn create_event_loop_waker(&self) -> Box<EventLoopWaker> {
        Box::new(WinitEventLoopWaker {
            proxy: self.waker.proxy.clone(),
        })
    }

    fn set_page_title(&self, _: BrowserId, title: Option<String>) {}

    fn status(&self, _: BrowserId, status: Option<String>) {}

    fn load_start(&self, _: BrowserId) {
        println!("load start");
    }

    fn load_end(&self, _: BrowserId) {
        println!("Load end");
    }

    fn history_changed(&self, _: BrowserId, history: Vec<LoadData>, current: usize) {}

    fn load_error(&self, _: BrowserId, _: NetError, _: String) {
        println!("load error");
    }

    fn head_parsed(&self, _: BrowserId) {
        println!("head parsed");
    }

    /// Has no effect on Android.
    fn set_cursor(&self, cursor: CursorKind) {}

    fn set_favicon(&self, _: BrowserId, _: ServoUrl) {}

    fn prepare_for_composite(&self, _width: usize, _height: usize) -> bool {
        true
    }

    /// Helper function to handle keyboard events.
    fn handle_key(
        &self,
        _: Option<BrowserId>,
        ch: Option<char>,
        key: Key,
        mods: constellation_msg::KeyModifiers,
    ) {
    }

    fn allow_navigation(&self, _: BrowserId, _: ServoUrl, response_chan: IpcSender<bool>) {}

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
