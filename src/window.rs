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
use std::usize;

pub struct NotificationWindow {
    layer_shell: Option<ZwlrLayerShellV1>,
    layer_surface: Option<ZwlrLayerSurfaceV1>,
    surface: Option<WlSurface>,
    buffer: Option<WlBuffer>,
    compositor: Option<WlCompositor>,
    shm: Option<WlShm>,
    pools: DoubleMemPool,
    display: Display,
    pub event_queue: EventQueue,
}
impl NotificationWindow {
    /// Create a new instance of `NotificationWindow`
    pub fn new(config: &WindowConfig) -> Self {
        // Connect to wayland server getting a Display
        // then derive a EventQueue, and an attached Display
        let display = Display::connect_to_env().unwrap();
        let mut event_queue = display.create_event_queue();
        let attached_display = (*display).clone().attach(event_queue.token());

        // Instantiate wayland globals
        let globals = GlobalManager::new(&attached_display);
        event_queue.sync_roundtrip(&mut (), |_, _, _| {}).unwrap();
        let compositor = globals
            .instantiate_exact::<wl_compositor::WlCompositor>(1)
            .unwrap();
        let shm = globals.instantiate_exact::<WlShm>(1).unwrap();
        let layer_shell = globals
            .instantiate_exact::<zwlr_layer_shell_v1::ZwlrLayerShellV1>(1)
            .unwrap();

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
        )
        .expect("Failed to create memory pool");

        // Return a instance of `NotificationWindow`
        Self {
            layer_shell: Some(layer_shell.detach()),
            layer_surface: Some(layer_surface.detach()),
            surface: Some(surface.detach()),
            compositor: Some(compositor.detach()),
            shm: Some(shm.detach()),
            buffer: None,
            display,
            event_queue,
            pools,
        }
    }

    /// Draws/renders the window using a wayland layer surface.
    // TODO: figure out lifetime and ownership problems so we can
    //       remove the unsafe block.
    pub fn draw(&mut self, msg: &str, config: &WindowConfig) {
        if let Some(pool) = self.pools.pool() {
            // Resize the pool to the size of the surface
            let width = config.size.width;
            let height = config.size.height;
            let bytes_per_px = 4;
            let size = (width * height * bytes_per_px) as usize;
            pool.resize(size).unwrap();

            // Do to lifetime and ownership complexities with creating a
            // cario surface buffer than can live long enough
            unsafe {
                // Create a intermediate buffer to the size of the surface
                let mut temp_buffer: Vec<u8> = vec![0; size];

                // Create a Cairo surface using the intermediate buffer
                let surface = ImageSurface::create_for_data_unsafe(
                    temp_buffer.as_mut_ptr(),
                    Format::ARgb32,
                    width as i32,
                    height as i32,
                    (width * bytes_per_px) as i32,
                )
                .expect("Failed to create Cairo surface");
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

                // Render the notification text
                let layout = Self::create_pango_layout(
                    &cr,
                    msg,
                    config.font_size,
                    (width as i32 - (config.margin.right + config.margin.left)) as u32,
                );
                cr.move_to(10.0, 10.0);
                pango_cairo::show_layout(&cr, &layout);

                // Copy the Cairo surface data to the Wayland buffer
                let mmap = pool.mmap();
                for (i, byte) in mmap.iter_mut().enumerate() {
                    *byte = temp_buffer[i];
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
                    surface.damage(0, 0, width as i32, height as i32);
                    surface.commit();
                }
            }
        }
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
