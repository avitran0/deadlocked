use std::{num::NonZeroU32, sync::Arc};

use egui::{Color32, FontData, FontDefinitions, Stroke, Style};
use egui_glow::glow::{self, HasContext as _};
use glutin::prelude::PossiblyCurrentGlContext;
use winit::platform::x11::{WindowAttributesExtX11, WindowType};

use crate::ui::color::Colors;

pub struct WindowContext {
    window: winit::window::Window,
    gl_context: glutin::context::PossiblyCurrentContext,
    _gl_display: glutin::display::Display,
    gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    glow: Arc<glow::Context>,
    egui_glow: egui_glow::EguiGlow,
    clear_color: Color32,
}

impl WindowContext {
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        overlay: bool,
        accent_color: egui::Color32,
    ) -> Self {
        use glutin::context::NotCurrentGlContext as _;
        use glutin::display::GetGlDisplay as _;
        use glutin::display::GlDisplay as _;
        use glutin::prelude::GlSurface as _;
        use winit::raw_window_handle::HasWindowHandle as _;

        let winit_window_builder = if overlay {
            winit::window::WindowAttributes::default()
                .with_decorations(false)
                .with_inner_size(winit::dpi::PhysicalSize::new(1, 1))
                .with_position(winit::dpi::PhysicalPosition::new(0, 0))
                .with_resizable(true)
                .with_transparent(true)
                .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
                .with_override_redirect(true)
                .with_x11_window_type(vec![WindowType::Tooltip])
                .with_title("deadlocked_overlay")
        } else {
            winit::window::WindowAttributes::default()
                .with_inner_size(winit::dpi::LogicalSize::new(750, 450))
                .with_title("deadlocked")
        };

        let config_template_builder = if overlay {
            glutin::config::ConfigTemplateBuilder::new()
                .prefer_hardware_accelerated(Some(true))
                .with_transparency(true)
        } else {
            glutin::config::ConfigTemplateBuilder::new()
                .prefer_hardware_accelerated(Some(true))
                .with_transparency(false)
        };

        let (mut window, gl_config) =
            glutin_winit::DisplayBuilder::new() // let glutin-winit helper crate handle the complex parts of opengl context creation
                .with_preference(glutin_winit::ApiPreference::FallbackEgl) // https://github.com/emilk/egui/issues/2520#issuecomment-1367841150
                .with_window_attributes(Some(winit_window_builder.clone()))
                .build(
                    event_loop,
                    config_template_builder,
                    |mut config_iterator| {
                        config_iterator.next().expect(
                            "failed to find a matching configuration for creating glutin config",
                        )
                    },
                )
                .expect("failed to create gl_config");
        let gl_display = gl_config.display();

        let raw_window_handle = window.as_ref().map(|w| {
            w.window_handle()
                .expect("failed to get window handle")
                .as_raw()
        });
        let context_attributes =
            glutin::context::ContextAttributesBuilder::new().build(raw_window_handle);
        let fallback_context_attributes = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::Gles(None))
            .build(raw_window_handle);
        let not_current_gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_config
                        .display()
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("failed to create context even with fallback attributes")
                })
        };

        // this is where the window is created, if it has not been created while searching for suitable gl_config
        let window = window.take().unwrap_or_else(|| {
            glutin_winit::finalize_window(event_loop, winit_window_builder.clone(), &gl_config)
                .expect("failed to finalize glutin window")
        });
        let (width, height): (u32, u32) = window.inner_size().into();
        let width = NonZeroU32::new(width).unwrap_or(NonZeroU32::MIN);
        let height = NonZeroU32::new(height).unwrap_or(NonZeroU32::MIN);
        let surface_attributes =
            glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                .build(
                    window
                        .window_handle()
                        .expect("failed to get window handle")
                        .as_raw(),
                    width,
                    height,
                );
        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &surface_attributes)
                .unwrap()
        };
        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

        gl_surface
            .set_swap_interval(&gl_context, glutin::surface::SwapInterval::DontWait)
            .unwrap();

        if overlay {
            window.set_cursor_hittest(false).unwrap();
            window.set_outer_position(winit::dpi::PhysicalPosition::new(0, 0));
        }

        let glow = unsafe {
            glow::Context::from_loader_function(|s| {
                let s = std::ffi::CString::new(s)
                    .expect("failed to construct C string from string for gl proc address");

                gl_display.get_proc_address(&s)
            })
        };

        let glow = Arc::new(glow);
        let mut egui_glow = egui_glow::EguiGlow::new(event_loop, glow.clone(), None, None, true);
        prep_ctx(&mut egui_glow.egui_ctx, accent_color);

        let clear_color = if overlay {
            Color32::TRANSPARENT
        } else {
            Color32::BLACK
        };

        Self {
            window,
            gl_context,
            _gl_display: gl_display,
            gl_surface,
            glow,
            egui_glow,
            clear_color,
        }
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn resize(&self, physical_size: winit::dpi::PhysicalSize<u32>) {
        use glutin::surface::GlSurface as _;
        let width = NonZeroU32::new(physical_size.width).unwrap_or(NonZeroU32::MIN);
        let height = NonZeroU32::new(physical_size.height).unwrap_or(NonZeroU32::MIN);
        self.gl_surface.resize(&self.gl_context, width, height);
    }

    pub fn swap_buffers(&self) -> glutin::error::Result<()> {
        use glutin::surface::GlSurface as _;
        self.gl_surface.swap_buffers(&self.gl_context)
    }

    pub fn make_current(&self) -> glutin::error::Result<()> {
        self.gl_context.make_current(&self.gl_surface)
    }

    pub fn process_event(&mut self, event: &winit::event::WindowEvent) -> egui_glow::EventResponse {
        self.egui_glow.on_window_event(&self.window, event)
    }

    pub fn process_modifier(&mut self, modifiers: egui::Modifiers, pressed: bool, repeat: bool) {
        self.egui_glow.egui_ctx.input_mut(|i| {
            i.events.push(egui::Event::Key {
                key: egui::Key::F35,
                physical_key: None,
                pressed,
                repeat,
                modifiers,
            });
        });
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn run(&mut self, func: impl FnMut(&egui::Context)) {
        self.egui_glow.run(&self.window, func);
    }

    pub fn clear(&self) {
        let [r, g, b, a] = self.clear_color.to_normalized_gamma_f32();
        unsafe {
            self.glow.clear_color(r, g, b, a);
            self.glow.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    pub fn paint(&mut self) {
        self.egui_glow.paint(&self.window);
    }
}

impl Drop for WindowContext {
    fn drop(&mut self) {
        self.egui_glow.destroy();
    }
}

fn prep_ctx(ctx: &mut egui::Context, accent_color: egui::Color32) {
    // add font
    let fira_sans = include_bytes!("../../resources/FiraSansIcons.ttf");
    let cs2_icons = include_bytes!("../../resources/CS2EquipmentIcons.ttf");
    let mut font_definitions = FontDefinitions::default();
    font_definitions.font_data.insert(
        String::from("fira_sans"),
        Arc::new(FontData::from_static(fira_sans)),
    );
    font_definitions.font_data.insert(
        String::from("cs2_icons"),
        Arc::new(FontData::from_static(cs2_icons)),
    );

    // insert into font definitions, so it gets chosen as default
    font_definitions
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, String::from("fira_sans"));
    font_definitions
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(0, String::from("cs2_icons"));

    ctx.set_fonts(font_definitions);

    ctx.style_mut_of(egui::Theme::Dark, |style| {
        gui_style(style, accent_color);
    });
}

fn gui_style(style: &mut Style, accent_color: egui::Color32) {
    style.interaction.selectable_labels = false;
    for font in style.text_styles.iter_mut() {
        font.1.size = 16.0;
    }
    //style.visuals.override_text_color = Some(Color32::WHITE);

    style.visuals.window_fill = Colors::BASE;
    style.visuals.panel_fill = Colors::BASE;
    style.visuals.extreme_bg_color = Colors::BACKDROP;

    let bg_stroke = Stroke::new(1.0, Colors::SUBTEXT);
    let fg_stroke = Stroke::new(1.0, Colors::TEXT);
    let dark_stroke = Stroke::new(1.0, Colors::BASE);

    style.visuals.selection.bg_fill = accent_color;
    style.visuals.selection.stroke = dark_stroke;

    style.visuals.widgets.active.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.active.bg_stroke = bg_stroke;
    style.visuals.widgets.active.fg_stroke = fg_stroke;
    style.visuals.widgets.active.weak_bg_fill = Colors::HIGHLIGHT;

    style.visuals.widgets.hovered.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.hovered.bg_stroke = bg_stroke;
    style.visuals.widgets.hovered.fg_stroke = fg_stroke;
    style.visuals.widgets.hovered.weak_bg_fill = Colors::HIGHLIGHT;

    style.visuals.widgets.inactive.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.inactive.fg_stroke = fg_stroke;
    style.visuals.widgets.inactive.weak_bg_fill = Colors::HIGHLIGHT;

    style.visuals.widgets.noninteractive.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.noninteractive.fg_stroke = fg_stroke;
    style.visuals.widgets.noninteractive.weak_bg_fill = Colors::HIGHLIGHT;

    style.visuals.widgets.open.bg_fill = Colors::HIGHLIGHT;
    style.visuals.widgets.open.bg_stroke = bg_stroke;
    style.visuals.widgets.open.fg_stroke = fg_stroke;
    style.visuals.widgets.open.weak_bg_fill = Colors::HIGHLIGHT;
}
