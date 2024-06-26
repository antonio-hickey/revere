use crate::{config::WindowConfig, error::RevereError};
use cairo::{Context, Format, ImageSurface};
use pango::{FontDescription, Layout};
use pangocairo::functions as pango_cairo;
use smithay_client_toolkit::{
    reexports::{
        client::{
            protocol::{
                wl_buffer::WlBuffer,
                wl_compositor::{self, WlCompositor},
                wl_shm::{Format as WlFormat, WlShm},
                wl_surface::WlSurface,
            },
            Display, EventQueue, GlobalManager,
        },
        protocols::wlr::unstable::layer_shell::v1::client::{
            zwlr_layer_shell_v1::{self, ZwlrLayerShellV1},
            zwlr_layer_surface_v1::{self, ZwlrLayerSurfaceV1},
        },
    },
    shm::DoubleMemPool,
};
use std::fs::File;

pub struct NotificationWindow {
    _layer_shell: Option<ZwlrLayerShellV1>,
    layer_surface: Option<ZwlrLayerSurfaceV1>,
    surface: Option<WlSurface>,
    buffer: Option<WlBuffer>,
    _compositor: Option<WlCompositor>,
    _shm: Option<WlShm>,
    pools: DoubleMemPool,
    display: Display,
    pub event_queue: EventQueue,
}
impl NotificationWindow {
    /// Create a new instance of `NotificationWindow`
    pub fn try_new(config: &WindowConfig) -> Result<Self, RevereError> {
        // Connect to wayland server getting a Display
        // then derive a EventQueue, and an attached Display
        let display = Display::connect_to_env()?;
        let mut event_queue = display.create_event_queue();
        let attached_display = (*display).clone().attach(event_queue.token());

        // Instantiate wayland globals
        let globals = GlobalManager::new(&attached_display);
        event_queue.sync_roundtrip(&mut (), |_, _, _| {})?;
        let compositor = globals.instantiate_exact::<wl_compositor::WlCompositor>(1)?;
        let shm = globals.instantiate_exact::<WlShm>(1)?;
        let layer_shell = globals.instantiate_exact::<zwlr_layer_shell_v1::ZwlrLayerShellV1>(1)?;

        // Derive a surface and layer surface from the server
        let surface = compositor.create_surface();
        let layer_surface = layer_shell.get_layer_surface(
            &surface,
            None, // put the surface on the default output
            zwlr_layer_shell_v1::Layer::Overlay,
            "my_notification".to_owned(),
        );

        // Configure the layer surface a bit and commit the changes
        layer_surface.set_size(config.size.width, config.size.height);
        layer_surface.set_anchor(config.placement.x.as_anchor() | config.placement.y.as_anchor());
        layer_surface.set_margin(
            config.margin.top,
            config.margin.right,
            config.margin.bottom,
            config.margin.left,
        );
        layer_surface.quick_assign(move |layer_surface, event, _| {
            if let zwlr_layer_surface_v1::Event::Configure { serial, .. } = event {
                layer_surface.ack_configure(serial);
            }
        });
        surface.commit();

        // Use a double buffering mechanism for smooth updates
        let pools = DoubleMemPool::new(
            shm.clone().into(),
            |_: smithay_client_toolkit::reexports::client::DispatchData| {},
        )?;

        // Return a instance of `NotificationWindow`
        let window = Self {
            _layer_shell: Some(layer_shell.detach()),
            layer_surface: Some(layer_surface.detach()),
            surface: Some(surface.detach()),
            _compositor: Some(compositor.detach()),
            _shm: Some(shm.detach()),
            buffer: None,
            display,
            event_queue,
            pools,
        };

        Ok(window)
    }

    /// Draws/renders the window using a wayland layer surface.
    pub fn draw(
        &mut self,
        msg: &str,
        thumbnail: &mut Option<File>,
        config: &WindowConfig,
    ) -> Result<(), RevereError> {
        if let Some(pool) = self.pools.pool() {
            // Resize the pool to the size of the surface
            let width = config.size.width;
            let height = config.size.height;
            let bytes_per_px = 4;
            let size = (width * height * bytes_per_px) as usize;
            pool.resize(size).unwrap();

            // Create a intermediate buffer to the size of the surface
            let temp_buffer: Vec<u8> = vec![0; size];

            // Create a Cairo surface using the intermediate buffer
            let mut surface = ImageSurface::create_for_data(
                temp_buffer,
                Format::ARgb32,
                width as i32,
                height as i32,
                (width * bytes_per_px) as i32,
            )?;

            // Handle the cairo surface context in a localized scope
            // to avoid any kind of ownership issues with the surface
            {
                let cr = Context::new(&surface).expect("some surface");

                // Perform cario drawing operations
                cr.set_source_rgb(
                    config.color.bg.red,
                    config.color.bg.green,
                    config.color.bg.blue,
                );
                cr.paint().ok(); // Fill the background
                cr.set_source_rgb(
                    config.color.fg.red,
                    config.color.fg.green,
                    config.color.fg.blue,
                );

                // Check if there's a thumbnail path provided and draw the PNG image
                if let Some(thumbnail_file) = thumbnail {
                    if let Ok(image_surface) = ImageSurface::create_from_png(thumbnail_file) {
                        // Scale the image down by half
                        image_surface.set_device_scale(2.0, 2.0);
                        let scaled_width = (image_surface.width() as f64) * 0.5;
                        let scaled_height = (image_surface.height() as f64) * 0.5;

                        // Draw the image
                        if let Err(e) = cr.set_source_surface(&image_surface, 0.0, 0.0) {
                            eprintln!("{e:?}");
                        }
                        cr.paint().expect("Failed to draw PNG image");

                        // Draw the image border
                        cr.rectangle(0.0, 0.0, scaled_width, scaled_height);
                        cr.set_source_rgba(0.0, 0.0, 0.0, 1.0);
                        cr.set_line_width(4.0);
                        if let Err(e) = cr.stroke() {
                            eprintln!("{e:?}");
                        }
                    }
                }

                // Render the notification text
                let layout = Self::create_pango_layout(
                    &cr,
                    msg,
                    config.font_size,
                    (width as i32 - 180) as u32,
                );
                cr.move_to(180.0, 40.0);
                pango_cairo::show_layout(&cr, &layout);

                // Draw the window border
                cr.rectangle(0.0, 0.0, width as f64, height as f64);
                cr.set_source_rgba(
                    config.border.color.red,
                    config.border.color.green,
                    config.border.color.blue,
                    config.border.alpha,
                );
                cr.set_line_width(config.border.width as f64);
                if let Err(e) = cr.stroke() {
                    eprintln!("{e:?}");
                }
            }

            // Copy the Cairo surface data to the Wayland buffer
            let mmap = pool.mmap();
            for (i, byte) in mmap.iter_mut().enumerate() {
                if let Some(surface_data_byte) = surface.data()?.get(i) {
                    *byte = *surface_data_byte;
                }
            }

            // Create a buffer from the memory pool for rendering the window
            self.buffer = Some(pool.buffer(
                0,
                width as i32,
                height as i32,
                (width * bytes_per_px) as i32,
                WlFormat::Argb8888,
            ));

            // Attach the buffer to the wayland surface, then damage the
            // surface to signal to wayland server to redraw (update) a surface
            // region, and finally commit the surface.
            if let Some(surface) = &self.surface {
                surface.attach(self.buffer.as_ref(), 0, 0);
                //surface.damage(0, 0, width as i32, height as i32);
                surface.commit();
            }
        }

        Ok(())
    }

    /// Flush the internal display buffer to the server socket.
    ///
    /// Non - blocking: If not all the requests could be written
    /// then returns RevereError::DisplayFlushError
    pub fn flush_display(&mut self) -> Result<(), RevereError> {
        // Destroy all the proxies attached to the event queue
        // before flushing the display, this is to clean up the
        // unsafe code block in `draw()` ensuring all memory safety.
        if let Some(surface) = &self.surface {
            surface.destroy();
        }
        if let Some(layer_surface) = &self.layer_surface {
            layer_surface.destroy();
        }
        if let Some(buffer) = &self.buffer {
            buffer.destroy();
        }

        // Flush the display
        self.display
            .flush()
            .map_err(|_| RevereError::DisplayFlushError)
    }

    /// Helper function to create a Pango layout for better text handeling like
    /// absolute size, text wrapping, and other stuff I'm not currently leveraging
    /// but may in the future like diff fonts, text alignment, and ellipsization.
    fn create_pango_layout(cr: &Context, text: &str, font_size: u8, max_width: u32) -> Layout {
        // Font stuff
        let mut font = FontDescription::from_string(&format!("sans {}", font_size));
        font.set_absolute_size(font_size as f64 * pango::SCALE as f64);

        // Layout stuff
        let layout = pango_cairo::create_layout(cr).expect("Cannot create pango layout");
        layout.set_font_description(Some(&font));
        layout.set_width(max_width as i32 * pango::SCALE);
        layout.set_wrap(pango::WrapMode::Word);
        layout.set_text(text);

        layout
    }
}
