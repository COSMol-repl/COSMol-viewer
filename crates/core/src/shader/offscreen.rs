use std::{ffi::CString, num::NonZeroU32, thread};
#[cfg(target_os = "windows")]
use std::{ffi::OsStr, num::NonZeroIsize, os::windows::ffi::OsStrExt};

use eframe::glow::{self, HasContext as _};
use egui_winit::winit::{
    event_loop::EventLoop,
    raw_window_handle::{
        HasWindowHandle as _, RawDisplayHandle, RawWindowHandle, WindowsDisplayHandle,
    },
    window::{Window, WindowAttributes},
};
use glam::Vec4;
use glutin::{
    config::{Config, ConfigTemplateBuilder, GlConfig},
    context::{ContextApi, ContextAttributesBuilder, GlProfile, Version},
    display::{Display, DisplayApiPreference, GetGlDisplay as _, GlDisplay as _},
    prelude::*,
    surface::{PbufferSurface, Surface, SurfaceAttributesBuilder},
};
use glutin_winit::{ApiPreference, DisplayBuilder};
use image::{ImageBuffer, Rgba};

use crate::{Scene, shader::CameraState};

use super::canvas::Shader;

pub struct ImageRenderer;

#[derive(Clone, Copy, Debug)]
pub enum ImageBackground {
    Scene,
    Color([f32; 4]),
}

struct OffscreenGl {
    gl: glow::Context,
    _backend: OffscreenBackend,
}

enum OffscreenBackend {
    Glutin {
        _context: glutin::context::PossiblyCurrentContext,
        _surface: Surface<PbufferSurface>,
        _window: Option<Window>,
        #[cfg(target_os = "windows")]
        _win32_window: Option<HiddenWin32Window>,
    },
    #[cfg(target_os = "linux")]
    RawEgl(RawEglContext),
}

#[cfg(target_os = "linux")]
struct RawEglContext {
    egl: glutin_egl_sys::egl::Egl,
    display: glutin_egl_sys::egl::types::EGLDisplay,
    context: glutin_egl_sys::egl::types::EGLContext,
    surface: glutin_egl_sys::egl::types::EGLSurface,
    _library: libloading::Library,
}

#[cfg(target_os = "windows")]
struct HiddenWin32Window {
    hwnd: windows_sys::Win32::Foundation::HWND,
    hinstance: windows_sys::Win32::Foundation::HINSTANCE,
    class_name: Vec<u16>,
}

#[cfg(target_os = "windows")]
unsafe impl Send for HiddenWin32Window {}

#[cfg(target_os = "windows")]
impl HiddenWin32Window {
    fn new(class_name: &str) -> Result<Self, String> {
        use windows_sys::Win32::{
            Foundation::HWND,
            System::LibraryLoader::GetModuleHandleW,
            UI::WindowsAndMessaging::{
                CS_OWNDC, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, RegisterClassW,
                WNDCLASSW, WS_DISABLED, WS_OVERLAPPED,
            },
        };

        let class_name = wide_null(class_name);
        let title = wide_null("cosmol_viewer_offscreen_context");
        let hinstance = unsafe { GetModuleHandleW(std::ptr::null()) };
        if hinstance.is_null() {
            return Err(format!(
                "GetModuleHandleW failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        let wc = WNDCLASSW {
            style: CS_OWNDC,
            lpfnWndProc: Some(DefWindowProcW),
            hInstance: hinstance,
            lpszClassName: class_name.as_ptr(),
            ..Default::default()
        };
        let atom = unsafe { RegisterClassW(&wc) };
        if atom == 0 {
            let error = std::io::Error::last_os_error();
            const ERROR_CLASS_ALREADY_EXISTS: i32 = 1410;
            if error.raw_os_error() != Some(ERROR_CLASS_ALREADY_EXISTS) {
                return Err(format!("RegisterClassW failed: {error}"));
            }
        }

        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPED | WS_DISABLED,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                1,
                1,
                std::ptr::null_mut::<std::ffi::c_void>() as HWND,
                std::ptr::null_mut(),
                hinstance,
                std::ptr::null(),
            )
        };
        if hwnd.is_null() {
            return Err(format!(
                "CreateWindowExW failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        Ok(Self {
            hwnd,
            hinstance,
            class_name,
        })
    }

    fn raw_window_handle(&self) -> Result<RawWindowHandle, String> {
        use egui_winit::winit::raw_window_handle::Win32WindowHandle;

        let hwnd = NonZeroIsize::new(self.hwnd as isize)
            .ok_or_else(|| "hidden Win32 window has null HWND".to_owned())?;
        let hinstance = NonZeroIsize::new(self.hinstance as isize);
        let mut handle = Win32WindowHandle::new(hwnd);
        handle.hinstance = hinstance;
        Ok(RawWindowHandle::Win32(handle))
    }
}

#[cfg(target_os = "windows")]
impl Drop for HiddenWin32Window {
    fn drop(&mut self) {
        use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyWindow, UnregisterClassW};

        unsafe {
            let _ = DestroyWindow(self.hwnd);
            let _ = UnregisterClassW(self.class_name.as_ptr(), self.hinstance);
        }
    }
}

#[cfg(target_os = "windows")]
fn wide_null(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

#[cfg(target_os = "windows")]
fn unique_wgl_class_name() -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("cosmol_viewer_offscreen_wgl_{}_{}", std::process::id(), id)
}

#[cfg(target_os = "linux")]
impl RawEglContext {
    const PLATFORM_SURFACELESS_MESA: u32 = 0x31DD;

    fn new(width: NonZeroU32, height: NonZeroU32) -> Result<(Self, glow::Context), String> {
        use glutin_egl_sys::egl;
        use std::ffi::c_void;

        offscreen_trace("loading libEGL");
        let library = unsafe {
            libloading::Library::new("libEGL.so.1")
                .or_else(|_| libloading::Library::new("libEGL.so"))
        }
        .map_err(|err| format!("could not load libEGL: {err}"))?;
        let egl = egl::Egl::load_with(|symbol| unsafe {
            library
                .get::<*const c_void>(symbol.as_bytes())
                .map(|address| *address)
                .unwrap_or(std::ptr::null())
        });

        offscreen_trace("requesting surfaceless EGL display");
        let display = unsafe {
            if egl.GetPlatformDisplay.is_loaded() {
                egl.GetPlatformDisplay(
                    Self::PLATFORM_SURFACELESS_MESA,
                    std::ptr::null_mut(),
                    [egl::NONE as isize].as_ptr(),
                )
            } else if egl.GetPlatformDisplayEXT.is_loaded() {
                egl.GetPlatformDisplayEXT(
                    Self::PLATFORM_SURFACELESS_MESA,
                    std::ptr::null_mut(),
                    [egl::NONE as i32].as_ptr(),
                )
            } else {
                return Err("libEGL exposes no platform-display function".to_owned());
            }
        };
        if display == egl::NO_DISPLAY {
            return Err(format!(
                "eglGetPlatformDisplay(EGL_PLATFORM_SURFACELESS_MESA) failed: {}",
                egl_error(&egl)
            ));
        }

        let mut major = 0;
        let mut minor = 0;
        offscreen_trace("initializing surfaceless EGL display");
        if unsafe { egl.Initialize(display, &mut major, &mut minor) } == egl::FALSE {
            return Err(format!(
                "eglInitialize(surfaceless) failed: {}",
                egl_error(&egl)
            ));
        }

        offscreen_trace("creating EGL context and pbuffer");
        let current = unsafe {
            Self::create_current_context(&egl, display, width, height, false).or_else(
                |desktop_error| {
                    Self::create_current_context(&egl, display, width, height, true).map_err(
                        |gles_error| {
                            format!("desktop OpenGL: {desktop_error}; OpenGL ES: {gles_error}")
                        },
                    )
                },
            )
        };
        let (context, surface) = match current {
            Ok(current) => current,
            Err(err) => {
                unsafe {
                    egl.Terminate(display);
                }
                return Err(err);
            }
        };

        let gl = unsafe {
            glow::Context::from_loader_function(|symbol| {
                let symbol = CString::new(symbol).expect("GL symbol contained NUL");
                egl.GetProcAddress(symbol.as_ptr()).cast()
            })
        };
        offscreen_trace("created EGL context and loaded GL functions");

        Ok((
            Self {
                egl,
                display,
                context,
                surface,
                _library: library,
            },
            gl,
        ))
    }

    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn create_current_context(
        egl: &glutin_egl_sys::egl::Egl,
        display: glutin_egl_sys::egl::types::EGLDisplay,
        width: NonZeroU32,
        height: NonZeroU32,
        use_gles: bool,
    ) -> Result<
        (
            glutin_egl_sys::egl::types::EGLContext,
            glutin_egl_sys::egl::types::EGLSurface,
        ),
        String,
    > {
        use glutin_egl_sys::egl;

        let api = if use_gles {
            egl::OPENGL_ES_API
        } else {
            egl::OPENGL_API
        };
        let renderable = if use_gles {
            egl::OPENGL_ES3_BIT
        } else {
            egl::OPENGL_BIT
        };
        if egl.BindAPI(api) == egl::FALSE {
            return Err(format!("eglBindAPI failed: {}", egl_error(egl)));
        }

        let config_attributes = [
            egl::SURFACE_TYPE as i32,
            egl::PBUFFER_BIT as i32,
            egl::RENDERABLE_TYPE as i32,
            renderable as i32,
            egl::RED_SIZE as i32,
            8,
            egl::GREEN_SIZE as i32,
            8,
            egl::BLUE_SIZE as i32,
            8,
            egl::ALPHA_SIZE as i32,
            8,
            egl::DEPTH_SIZE as i32,
            24,
            egl::NONE as i32,
        ];
        let mut config = std::ptr::null();
        let mut config_count = 0;
        if egl.ChooseConfig(
            display,
            config_attributes.as_ptr(),
            &mut config,
            1,
            &mut config_count,
        ) == egl::FALSE
        {
            return Err(format!("eglChooseConfig failed: {}", egl_error(egl)));
        }
        if config_count == 0 || config.is_null() {
            return Err("eglChooseConfig returned no matching pbuffer config".to_owned());
        }

        let surface_attributes = [
            egl::WIDTH as i32,
            width.get() as i32,
            egl::HEIGHT as i32,
            height.get() as i32,
            egl::NONE as i32,
        ];
        let surface = egl.CreatePbufferSurface(display, config, surface_attributes.as_ptr());
        if surface == egl::NO_SURFACE {
            return Err(format!(
                "eglCreatePbufferSurface failed: {}",
                egl_error(egl)
            ));
        }

        let context_attributes = if use_gles {
            vec![egl::CONTEXT_CLIENT_VERSION as i32, 3, egl::NONE as i32]
        } else {
            vec![
                egl::CONTEXT_MAJOR_VERSION as i32,
                3,
                egl::CONTEXT_MINOR_VERSION as i32,
                3,
                egl::CONTEXT_OPENGL_PROFILE_MASK as i32,
                egl::CONTEXT_OPENGL_CORE_PROFILE_BIT as i32,
                egl::NONE as i32,
            ]
        };
        let context = egl.CreateContext(
            display,
            config,
            egl::NO_CONTEXT,
            context_attributes.as_ptr(),
        );
        if context == egl::NO_CONTEXT {
            let err = egl_error(egl);
            egl.DestroySurface(display, surface);
            return Err(format!("eglCreateContext failed: {err}"));
        }
        if egl.MakeCurrent(display, surface, surface, context) == egl::FALSE {
            let err = egl_error(egl);
            egl.DestroyContext(display, context);
            egl.DestroySurface(display, surface);
            return Err(format!("eglMakeCurrent failed: {err}"));
        }

        Ok((context, surface))
    }
}

#[cfg(target_os = "linux")]
impl Drop for RawEglContext {
    fn drop(&mut self) {
        use glutin_egl_sys::egl;

        unsafe {
            offscreen_trace("releasing EGL context");
            self.egl.MakeCurrent(
                self.display,
                egl::NO_SURFACE,
                egl::NO_SURFACE,
                egl::NO_CONTEXT,
            );
            self.egl.DestroyContext(self.display, self.context);
            self.egl.DestroySurface(self.display, self.surface);
            self.egl.Terminate(self.display);
            offscreen_trace("released EGL context");
        }
    }
}

#[cfg(target_os = "linux")]
fn egl_error(egl: &glutin_egl_sys::egl::Egl) -> String {
    format!("EGL error 0x{:04x}", unsafe { egl.GetError() })
}

impl ImageRenderer {
    pub fn render(
        scene: &Scene,
        width: u32,
        height: u32,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String> {
        Self::render_with_background(scene, width, height, ImageBackground::Scene)
    }

    pub fn render_with_background(
        scene: &Scene,
        width: u32,
        height: u32,
        background: ImageBackground,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String> {
        let scene = scene.clone();
        thread::Builder::new()
            .name("cosmol_viewer_offscreen_render".to_owned())
            .spawn(move || {
                offscreen_trace("creating offscreen GL backend");
                let mut gl = OffscreenGl::new()?;
                offscreen_trace("created offscreen GL backend");
                let image = gl.render(&scene, width, height, background)?;
                offscreen_trace("completed offscreen render");
                Ok(image)
            })
            .map_err(|err| format!("failed to start offscreen render thread: {err}"))?
            .join()
            .map_err(|_| "offscreen render thread panicked".to_owned())?
    }

    pub fn save_png(
        scene: &Scene,
        path: impl AsRef<std::path::Path>,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        Self::save_png_with_background(scene, path, width, height, ImageBackground::Scene)
    }

    pub fn save_png_with_background(
        scene: &Scene,
        path: impl AsRef<std::path::Path>,
        width: u32,
        height: u32,
        background: ImageBackground,
    ) -> Result<(), String> {
        let image = Self::render_with_background(scene, width, height, background)?;
        image.save(path).map_err(|err| err.to_string())
    }

    pub fn render_png_bytes(scene: &Scene, width: u32, height: u32) -> Result<Vec<u8>, String> {
        Self::render_png_bytes_with_background(scene, width, height, ImageBackground::Scene)
    }

    pub fn render_png_bytes_with_background(
        scene: &Scene,
        width: u32,
        height: u32,
        background: ImageBackground,
    ) -> Result<Vec<u8>, String> {
        let image = Self::render_with_background(scene, width, height, background)?;
        let mut bytes = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .map_err(|err| err.to_string())?;
        Ok(bytes)
    }
}

impl OffscreenGl {
    fn new() -> Result<Self, String> {
        let width = NonZeroU32::new(1).expect("1 is non-zero");
        let height = NonZeroU32::new(1).expect("1 is non-zero");

        #[cfg(target_os = "linux")]
        if linux_is_displayless() {
            return Self::new_headless_egl(width, height);
        }

        #[cfg(target_os = "windows")]
        match Self::new_headless_wgl(width, height) {
            Ok(gl) => return Ok(gl),
            Err(err) => {
                eprintln!(
                    "[WARN] Headless WGL offscreen initialization failed; falling back to winit bootstrap: {err}"
                );
            }
        }

        Self::new_with_winit(width, height)
    }

    fn new_with_winit(width: NonZeroU32, height: NonZeroU32) -> Result<Self, String> {
        let event_loop = offscreen_event_loop_builder().build().map_err(|err| {
            format!("{err}. Offscreen rendering could not create its GL bootstrap event loop.")
        })?;
        let template = offscreen_config_template_builder(width, height);

        let (window, gl_config) = DisplayBuilder::new()
            .with_preference(ApiPreference::FallbackEgl)
            .with_window_attributes(bootstrap_window_attributes())
            .build(&event_loop, template, |configs| {
                configs
                    .max_by_key(|config| config.num_samples())
                    .expect("no GL configs found")
            })
            .map_err(|err| err.to_string())?;

        let raw_window_handle = window
            .as_ref()
            .and_then(|window| window.window_handle().ok())
            .map(|handle| handle.as_raw());

        Self::new_from_config(width, height, gl_config, raw_window_handle, window)
    }

    #[cfg(target_os = "windows")]
    fn new_headless_wgl(width: NonZeroU32, height: NonZeroU32) -> Result<Self, String> {
        let window = HiddenWin32Window::new(&unique_wgl_class_name())?;
        let raw_window_handle = window.raw_window_handle()?;
        let raw_display = RawDisplayHandle::Windows(WindowsDisplayHandle::new());
        let gl_display = unsafe {
            Display::new(
                raw_display,
                DisplayApiPreference::Wgl(Some(raw_window_handle)),
            )
            .map_err(|err| err.to_string())?
        };
        let template = offscreen_config_template_builder(width, height).build();
        let gl_config = unsafe {
            gl_display
                .find_configs(template)
                .map_err(|err| err.to_string())?
                .max_by_key(|config| config.num_samples())
                .ok_or_else(|| "WGL display returned no GL configs".to_owned())?
        };

        Self::new_from_config_with_win32_window(
            width,
            height,
            gl_config,
            Some(raw_window_handle),
            window,
        )
        .map_err(|err| format!("WGL display: {err}"))
    }

    #[cfg(target_os = "linux")]
    fn new_headless_egl(width: NonZeroU32, height: NonZeroU32) -> Result<Self, String> {
        use egui_winit::winit::raw_window_handle::{RawDisplayHandle, XlibDisplayHandle};
        use glutin::{
            api::egl::{device::Device, display::Display as EglDisplay},
            display::{Display, DisplayApiPreference},
        };

        let mut errors = Vec::new();

        match RawEglContext::new(width, height) {
            Ok((context, gl)) => {
                return Ok(Self {
                    gl,
                    _backend: OffscreenBackend::RawEgl(context),
                });
            }
            Err(err) => errors.push(format!("EGL surfaceless display: {err}")),
        }

        match Device::query_devices() {
            Ok(devices) => {
                for device in devices {
                    let egl_display = match unsafe { EglDisplay::with_device(&device, None) } {
                        Ok(display) => display,
                        Err(err) => {
                            errors.push(format!("EGL device display: {err}"));
                            continue;
                        }
                    };
                    let gl_display = Display::Egl(egl_display);
                    match Self::new_from_headless_display(
                        width,
                        height,
                        gl_display,
                        "EGL device display",
                    ) {
                        Ok(gl) => return Ok(gl),
                        Err(err) => errors.push(err),
                    }
                }
            }
            Err(err) => errors.push(format!("{err}. Headless EGL device enumeration failed.")),
        }

        let raw_display = RawDisplayHandle::Xlib(XlibDisplayHandle::new(None, 0));
        match unsafe { Display::new(raw_display, DisplayApiPreference::Egl) } {
            Ok(gl_display) => {
                match Self::new_from_headless_display(
                    width,
                    height,
                    gl_display,
                    "EGL default display",
                ) {
                    Ok(gl) => return Ok(gl),
                    Err(err) => errors.push(err),
                }
            }
            Err(err) => errors.push(format!("EGL default display: {err}")),
        }

        Err(format!(
            "Headless EGL could not create an offscreen display{}",
            if errors.is_empty() {
                ".".to_owned()
            } else {
                format!(": {}", errors.join("; "))
            }
        ))
    }

    #[cfg(target_os = "linux")]
    fn new_from_headless_display(
        width: NonZeroU32,
        height: NonZeroU32,
        gl_display: glutin::display::Display,
        label: &str,
    ) -> Result<Self, String> {
        let template = offscreen_config_template_builder(width, height).build();
        let gl_config = match unsafe { gl_display.find_configs(template) } {
            Ok(configs) => configs.max_by_key(|config| config.num_samples()),
            Err(err) => return Err(format!("{label}: {err}")),
        };

        match gl_config {
            Some(gl_config) => Self::new_from_config(width, height, gl_config, None, None)
                .map_err(|err| format!("{label}: {err}")),
            None => Err(format!("{label}: no GL configs found")),
        }
    }

    fn new_from_config(
        width: NonZeroU32,
        height: NonZeroU32,
        gl_config: Config,
        raw_window_handle: Option<RawWindowHandle>,
        window: Option<Window>,
    ) -> Result<Self, String> {
        let gl_display = gl_config.display();

        let context_attributes = ContextAttributesBuilder::new()
            .with_profile(GlProfile::Core)
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .build(raw_window_handle);

        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(raw_window_handle);

        let not_current = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .or_else(|_| gl_display.create_context(&gl_config, &fallback_context_attributes))
                .map_err(|err| err.to_string())?
        };

        let surface_attributes = SurfaceAttributesBuilder::<PbufferSurface>::new()
            .with_single_buffer(true)
            .build(width, height);
        let surface = unsafe {
            gl_display
                .create_pbuffer_surface(&gl_config, &surface_attributes)
                .map_err(|err| err.to_string())?
        };
        let context = not_current
            .make_current(&surface)
            .map_err(|err| err.to_string())?;

        let gl = unsafe {
            glow::Context::from_loader_function(|symbol| {
                let symbol = CString::new(symbol).expect("GL symbol contained NUL");
                gl_display.get_proc_address(&symbol)
            })
        };

        Ok(Self {
            gl,
            _backend: OffscreenBackend::Glutin {
                _context: context,
                _surface: surface,
                _window: window,
                #[cfg(target_os = "windows")]
                _win32_window: None,
            },
        })
    }

    #[cfg(target_os = "windows")]
    fn new_from_config_with_win32_window(
        width: NonZeroU32,
        height: NonZeroU32,
        gl_config: Config,
        raw_window_handle: Option<RawWindowHandle>,
        win32_window: HiddenWin32Window,
    ) -> Result<Self, String> {
        let mut gl = Self::new_from_config(width, height, gl_config, raw_window_handle, None)?;
        match &mut gl._backend {
            OffscreenBackend::Glutin { _win32_window, .. } => {
                *_win32_window = Some(win32_window);
            }
        }
        Ok(gl)
    }

    fn render(
        &mut self,
        scene: &Scene,
        width: u32,
        height: u32,
        background: ImageBackground,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String> {
        if width == 0 || height == 0 {
            return Err("width and height must be non-zero".to_owned());
        }

        let gl = &self.gl;
        offscreen_trace("creating scene shader");
        let mut shader =
            Shader::new(gl, scene).ok_or_else(|| "failed to initialize shader".to_owned())?;
        offscreen_trace("created scene shader");
        if let ImageBackground::Color(background_color) = background {
            shader.set_background_color(Vec4::from_array(background_color));
        }
        let camera_state = scene.camera_state.unwrap_or_else(CameraState::default);
        let aspect_ratio = width as f32 / height as f32;
        let samples = offscreen_samples(gl);

        unsafe {
            if samples == 1 {
                return render_single_sample(
                    gl,
                    &mut shader,
                    &camera_state,
                    aspect_ratio,
                    width,
                    height,
                );
            }

            let msaa_framebuffer = gl.create_framebuffer().map_err(|err| err.to_string())?;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(msaa_framebuffer));

            let msaa_color = gl.create_renderbuffer().map_err(|err| err.to_string())?;
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(msaa_color));
            gl.renderbuffer_storage_multisample(
                glow::RENDERBUFFER,
                samples,
                glow::RGBA8,
                width as i32,
                height as i32,
            );
            gl.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::RENDERBUFFER,
                Some(msaa_color),
            );

            let msaa_depth = gl.create_renderbuffer().map_err(|err| err.to_string())?;
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(msaa_depth));
            gl.renderbuffer_storage_multisample(
                glow::RENDERBUFFER,
                samples,
                glow::DEPTH_COMPONENT24,
                width as i32,
                height as i32,
            );
            gl.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::DEPTH_ATTACHMENT,
                glow::RENDERBUFFER,
                Some(msaa_depth),
            );

            let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
            if status != glow::FRAMEBUFFER_COMPLETE {
                gl.delete_renderbuffer(msaa_depth);
                gl.delete_renderbuffer(msaa_color);
                gl.delete_framebuffer(msaa_framebuffer);
                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                return Err(format!(
                    "offscreen MSAA framebuffer is incomplete: 0x{status:x}"
                ));
            }

            gl.viewport(0, 0, width as i32, height as i32);
            shader.paint(gl, aspect_ratio, &camera_state, true);
            gl.finish();

            let resolve_framebuffer = gl.create_framebuffer().map_err(|err| err.to_string())?;
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(resolve_framebuffer));

            let resolve_texture = gl.create_texture().map_err(|err| err.to_string())?;
            gl.bind_texture(glow::TEXTURE_2D, Some(resolve_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(resolve_texture),
                0,
            );

            let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
            if status != glow::FRAMEBUFFER_COMPLETE {
                gl.delete_texture(resolve_texture);
                gl.delete_framebuffer(resolve_framebuffer);
                gl.delete_renderbuffer(msaa_depth);
                gl.delete_renderbuffer(msaa_color);
                gl.delete_framebuffer(msaa_framebuffer);
                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                return Err(format!(
                    "offscreen resolve framebuffer is incomplete: 0x{status:x}"
                ));
            }

            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(msaa_framebuffer));
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(resolve_framebuffer));
            gl.blit_framebuffer(
                0,
                0,
                width as i32,
                height as i32,
                0,
                0,
                width as i32,
                height as i32,
                glow::COLOR_BUFFER_BIT,
                glow::NEAREST,
            );

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(resolve_framebuffer));
            let mut pixels = vec![0_u8; width as usize * height as usize * 4];
            gl.read_pixels(
                0,
                0,
                width as i32,
                height as i32,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(Some(&mut pixels)),
            );

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.delete_texture(resolve_texture);
            gl.delete_framebuffer(resolve_framebuffer);
            gl.delete_renderbuffer(msaa_depth);
            gl.delete_renderbuffer(msaa_color);
            gl.delete_framebuffer(msaa_framebuffer);

            flip_rgba_rows(&mut pixels, width as usize, height as usize);

            ImageBuffer::from_raw(width, height, pixels)
                .ok_or_else(|| "failed to build image buffer from GL pixels".to_owned())
        }
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn render_single_sample(
    gl: &glow::Context,
    shader: &mut Shader,
    camera_state: &CameraState,
    aspect_ratio: f32,
    width: u32,
    height: u32,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String> {
    offscreen_trace("creating single-sample framebuffer");
    let framebuffer = gl.create_framebuffer().map_err(|err| err.to_string())?;
    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));

    let color = gl.create_texture().map_err(|err| err.to_string())?;
    gl.bind_texture(glow::TEXTURE_2D, Some(color));
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::RGBA8 as i32,
        width as i32,
        height as i32,
        0,
        glow::RGBA,
        glow::UNSIGNED_BYTE,
        glow::PixelUnpackData::Slice(None),
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MIN_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MAG_FILTER,
        glow::LINEAR as i32,
    );
    gl.framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        glow::COLOR_ATTACHMENT0,
        glow::TEXTURE_2D,
        Some(color),
        0,
    );

    let depth = gl.create_renderbuffer().map_err(|err| err.to_string())?;
    gl.bind_renderbuffer(glow::RENDERBUFFER, Some(depth));
    gl.renderbuffer_storage(
        glow::RENDERBUFFER,
        glow::DEPTH_COMPONENT24,
        width as i32,
        height as i32,
    );
    gl.framebuffer_renderbuffer(
        glow::FRAMEBUFFER,
        glow::DEPTH_ATTACHMENT,
        glow::RENDERBUFFER,
        Some(depth),
    );

    let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
    if status != glow::FRAMEBUFFER_COMPLETE {
        gl.delete_renderbuffer(depth);
        gl.delete_texture(color);
        gl.delete_framebuffer(framebuffer);
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        return Err(format!(
            "offscreen single-sample framebuffer is incomplete: 0x{status:x}"
        ));
    }

    offscreen_trace("painting single-sample framebuffer");
    gl.viewport(0, 0, width as i32, height as i32);
    shader.paint(gl, aspect_ratio, camera_state, false);
    offscreen_trace("finishing single-sample framebuffer");
    gl.finish();

    offscreen_trace("reading single-sample framebuffer");
    let mut pixels = vec![0_u8; width as usize * height as usize * 4];
    gl.read_pixels(
        0,
        0,
        width as i32,
        height as i32,
        glow::RGBA,
        glow::UNSIGNED_BYTE,
        glow::PixelPackData::Slice(Some(&mut pixels)),
    );

    offscreen_trace("releasing single-sample framebuffer");
    gl.bind_framebuffer(glow::FRAMEBUFFER, None);
    gl.delete_renderbuffer(depth);
    gl.delete_texture(color);
    gl.delete_framebuffer(framebuffer);

    flip_rgba_rows(&mut pixels, width as usize, height as usize);
    ImageBuffer::from_raw(width, height, pixels)
        .ok_or_else(|| "failed to build image buffer from GL pixels".to_owned())
}

fn offscreen_trace(stage: &str) {
    if std::env::var_os("COSMOL_VIEWER_OFFSCREEN_TRACE").is_none() {
        return;
    }

    use std::io::Write as _;
    let mut stderr = std::io::stderr().lock();
    let _ = writeln!(stderr, "[cosmol_viewer offscreen] {stage}");
    let _ = stderr.flush();
}

fn offscreen_samples(gl: &glow::Context) -> i32 {
    let requested = std::env::var("COSMOL_VIEWER_OFFSCREEN_SAMPLES")
        .ok()
        .and_then(|value| value.parse::<i32>().ok());

    let default_samples = if software_gl_requested() || software_gl_renderer(gl) {
        1
    } else {
        4
    };
    let requested = requested.unwrap_or(default_samples).clamp(1, 16);

    unsafe {
        let max_samples = gl.get_parameter_i32(glow::MAX_SAMPLES).max(1);
        requested.min(max_samples)
    }
}

fn software_gl_requested() -> bool {
    std::env::var("LIBGL_ALWAYS_SOFTWARE")
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn software_gl_renderer(gl: &glow::Context) -> bool {
    let renderer = unsafe { gl.get_parameter_string(glow::RENDERER) }.to_ascii_lowercase();
    renderer.contains("llvmpipe")
        || renderer.contains("softpipe")
        || renderer.contains("swrast")
        || renderer.contains("software rasterizer")
}

fn offscreen_config_template_builder(
    width: NonZeroU32,
    height: NonZeroU32,
) -> ConfigTemplateBuilder {
    ConfigTemplateBuilder::new()
        .with_depth_size(24)
        .with_alpha_size(8)
        .with_pbuffer_sizes(width, height)
}

fn offscreen_event_loop_builder() -> egui_winit::winit::event_loop::EventLoopBuilder<()> {
    let mut builder = EventLoop::builder();
    #[cfg(target_family = "windows")]
    {
        use egui_winit::winit::platform::windows::EventLoopBuilderExtWindows;
        builder.with_any_thread(true);
    }
    #[cfg(feature = "wayland")]
    {
        use egui_winit::winit::platform::wayland::EventLoopBuilderExtWayland;
        builder.with_any_thread(true);
    }
    #[cfg(feature = "x11")]
    {
        use egui_winit::winit::platform::x11::EventLoopBuilderExtX11;
        builder.with_any_thread(true);
    }
    builder
}

#[cfg(target_os = "linux")]
fn linux_is_displayless() -> bool {
    std::env::var_os("WAYLAND_DISPLAY").is_none()
        && std::env::var_os("WAYLAND_SOCKET").is_none()
        && std::env::var_os("DISPLAY").is_none()
}

fn bootstrap_window_attributes() -> Option<WindowAttributes> {
    if cfg!(target_os = "windows") {
        Some(
            WindowAttributes::default()
                .with_visible(false)
                .with_title("cosmol_viewer_offscreen_context"),
        )
    } else {
        None
    }
}

fn flip_rgba_rows(pixels: &mut [u8], width: usize, height: usize) {
    let stride = width * 4;
    for y in 0..(height / 2) {
        let top = y * stride;
        let bottom = (height - 1 - y) * stride;
        for x in 0..stride {
            pixels.swap(top + x, bottom + x);
        }
    }
}
