use cairo::{Context, FontSlant, FontWeight, Format, ImageSurface};
use smithay_client_toolkit::{
    reexports::{
        client::{
            protocol::{
                wl_compositor,
                wl_shm::{Format as WlFormat, WlShm},
                wl_surface::WlSurface,
            },
            Display, EventQueue, GlobalManager,
        },
        protocols::wlr::unstable::layer_shell::v1::client::{
            zwlr_layer_shell_v1, zwlr_layer_surface_v1,
        },
    },
    shm::DoubleMemPool,
};

use crate::error::RevereError;

pub struct NotificationWindow {
    _layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    surface: Option<WlSurface>,
    pools: DoubleMemPool,
    display: Display,
    pub event_queue: EventQueue,
}
impl NotificationWindow {
    /// Create a new instance of `NotificationWindow`
    pub fn new() -> Self {
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
        layer_surface.set_size(200, 100);
        layer_surface
            .set_anchor(zwlr_layer_surface_v1::Anchor::Top | zwlr_layer_surface_v1::Anchor::Right);
        layer_surface.set_margin(10, 10, 0, 0);
        layer_surface.quick_assign(move |layer_surface, event, _| {
            if let zwlr_layer_surface_v1::Event::Configure { serial, .. } = event {
                layer_surface.ack_configure(serial);
            }
        });
        surface.commit();

        // Use a double buffering mechanism for smooth updates
        let pools = DoubleMemPool::new(
            shm.into(),
            |_: smithay_client_toolkit::reexports::client::DispatchData| {},
        )
        .expect("Failed to create memory pool");

        // Return a instance of `NotificationWindow`
        Self {
            _layer_surface: Some(layer_surface.detach()),
            surface: Some(surface.detach()),
            display,
            event_queue,
            pools,
        }
    }

    /// Draws/renders the window using a wayland layer surface.
    // TODO: figure out lifetime and ownership problems so we can
    //       remove the unsafe block.
    pub fn draw(&mut self, msg: &str) {
        if let Some(pool) = self.pools.pool() {
            // Resize the pool to the size of the surface
            let width = 200;
            let height = 100;
            let bytes_per_px = 4;
            let size = width * height * bytes_per_px;
            pool.resize(size).unwrap();

            // Do to lifetime and ownership complexities with creating a
            // cario surface buffer than can live long enough
            unsafe {
                // Create a intermediate buffer to the size of the surface
                let mut temp_buffer: Vec<u8> = vec![0; width * height * 4];

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
                cr.set_source_rgb(1.0, 1.0, 1.0); // White text
                cr.paint().ok(); // Fill the background
                cr.set_source_rgb(0.0, 0.0, 0.0); // Black text
                cr.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
                cr.set_font_size(20.0);
                cr.move_to(10.0, 50.0);
                cr.show_text(msg).expect("Failed to draw text");

                // Copy the Cairo surface data to the Wayland buffer
                let mmap = pool.mmap();
                for (i, byte) in mmap.iter_mut().enumerate() {
                    *byte = temp_buffer[i];
                }

                // Create a buffer from the memory pool for rendering the window
                let buffer = pool.buffer(
                    0,
                    width as i32,
                    height as i32,
                    (width * bytes_per_px) as i32,
                    WlFormat::Argb8888,
                );

                // Attach the buffer to the wayland surface, then damage the
                // surface to signal to wayland server to redraw (update) a surface
                // region, and finally commit the surface.
                if let Some(surface) = &self.surface {
                    surface.attach(Some(&buffer), 0, 0);
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
    pub fn flush_display(&self) -> Result<(), RevereError> {
        self.display
            .flush()
            .map_err(|_| RevereError::DisplayFlushError)
    }
}
