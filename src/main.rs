use std::sync::{Arc, RwLock};

use gtk::{
    cairo::Context,
    gdk::{Event, EventMask, EventMotion, EventScroll, ScrollDirection},
    gdk_pixbuf::Pixbuf,
    glib::{self, translate::ToGlibPtr},
    prelude::*,
    Align, BaselinePosition, DrawingArea, EventBox, FlowBox, Label, Orientation, Statusbar,
};
use std::future::Future;

pub fn thread_context() -> glib::MainContext {
    glib::MainContext::thread_default().unwrap_or_else(|| {
        let ctx = glib::MainContext::new();
        unsafe {
            glib::ffi::g_main_context_push_thread_default(ctx.to_glib_none().0);
        }
        ctx
    })
}

mod utils;
use crate::utils::map::{meters_to_lon_lat, pixels_to_image_coordinates, pixels_to_meters};

mod backend;
use crate::backend::predict;

pub struct MapState {
    pan_position: RwLock<(f64, f64)>,
    pan_start_pos: RwLock<(f64, f64)>,
    pan_delta: RwLock<(f64, f64)>,
    panning: RwLock<bool>,

    zoom_level: RwLock<f32>,

    mouse_position: RwLock<(f64, f64)>,
}

fn event_box_hook_up(evt_box: &EventBox, statusbar: Arc<Statusbar>, map_state: Arc<MapState>) {
    {
        let map_state = map_state.clone();
        evt_box.connect("scroll-event", false, move |args| {
            let evt_box = args[0].get::<EventBox>().ok()?;
            let event = args[1].get::<Event>().ok()?;
            let scroll_event = event.downcast_ref::<EventScroll>()?;

            let direction = match scroll_event.direction() {
                ScrollDirection::Up => 0.1,
                ScrollDirection::Down => -0.1,
                _ => 0.0,
            };

            let mut zoom_level = map_state.zoom_level.write().unwrap();
            *zoom_level = (*zoom_level + direction).max(0.1).min(5.0);
            println!("Zoom level: {}", zoom_level);
            evt_box.child()?.queue_draw();

            Some(true.to_value())
        });
    }

    {
        let map_state = map_state.clone();

        evt_box.connect_button_press_event(move |_evt_box, event| {
            // We're panning
            let mut panning = map_state.panning.write().unwrap();
            *panning = true;

            // This is where we started (needed to calculate user's mouse
            // motion, the position delta)
            let mut pan_start_pos = map_state.pan_start_pos.write().unwrap();
            *pan_start_pos = event.position();

            Inhibit(false)
        });
    }

    {
        let map_state = map_state.clone();

        evt_box.connect_button_release_event(move |_evt_box, event| {
            let mut pan_delta = map_state.pan_delta.write().unwrap();
            // Update the position of the image
            let mut pan_position = map_state.pan_position.write().unwrap();
            *pan_position = (pan_position.0 + pan_delta.0, pan_position.1 + pan_delta.1);
            // Then reset delta to zero for the next drag (and for in between
            // drags, since the position of the image has been updated we nor
            // longer need to add anything to it to make it match the dragged
            // location, otherwise we're double counting!)
            let did_pan = !(pan_delta.0 == 0.0 && pan_delta.1 == 0.0);
            *pan_delta = (0.0, 0.0);

            let mut panning = map_state.panning.write().unwrap();
            *panning = false;

            // Don't need these any more, and if we're gonna read from some of
            // them later, in the lat-lon code, then we need to drop the write
            // handles for it to not block
            drop(panning);
            drop(pan_position);
            drop(pan_delta);

            // Calculate the lattitude and longitude (and eventually run the
            // model) on a new thread, that's popped up in a dialog to display
            // the results and indicate a computation is occuring
            let pos = event.position();
            if !did_pan {
                let map_state = map_state.clone();
                std::thread::spawn(move || {
                    thread_context().block_on(predict(pos, map_state));
                });
            }
            Inhibit(false)
        });
    }

    {
        let map_state = map_state.clone();

        evt_box.connect_motion_notify_event(move |evt_box, motion_event: &EventMotion| {
            let mut mouse_position = map_state.mouse_position.write().unwrap();
            *mouse_position = motion_event.position();

            if *map_state.panning.read().unwrap() {
                let mut pan_delta = map_state.pan_delta.write().unwrap();
                let pan_start_pos = map_state.pan_start_pos.read().unwrap();
                // How did the user drag the mouse from the start position?
                *pan_delta = (
                    motion_event.position().0 - pan_start_pos.0,
                    motion_event.position().1 - pan_start_pos.1,
                );

                evt_box.child().unwrap().queue_draw();
            }

            drop(mouse_position);

            let meters = pixels_to_meters(
                motion_event.position().0,
                motion_event.position().1,
                &map_state,
            );
            let apos = pixels_to_image_coordinates(
                motion_event.position().0,
                motion_event.position().1,
                &map_state,
            );
            let lonlat = meters_to_lon_lat(meters.0, meters.1);

            statusbar.push(
                statusbar.context_id("cursor coord data"),
                format!(
                    "ABS: ({:.2}, {:.2}), MERC: ({:.2}, {:.2}), LON/LAT: ({:.2}, {:.2})",
                    apos.0, apos.1, meters.0, meters.1, lonlat.0, lonlat.1
                )
                .as_ref(),
            );
            Inhibit(false)
        });
    }
}

fn draw_map(map_state: Arc<MapState>) -> impl Fn(&DrawingArea, &Context) -> Inhibit {
    let pixbuf =
        Pixbuf::from_file("assets/rawmap.png").expect("Can't load image necessary for application");
    return move |_, cr| {
        let zoom_level = map_state.zoom_level.read().unwrap();
        cr.scale(*zoom_level as f64, *zoom_level as f64);

        let pan_position = map_state.pan_position.read().unwrap();
        let pan_delta = map_state.pan_delta.read().unwrap();
        // Reproduce the user's dragging motion relative to the actual position
        // of the image so that it doesn't reset to the user's cursor every
        // time they drag
        cr.set_source_pixbuf(
            &pixbuf,
            (pan_position.0 + pan_delta.0) / *zoom_level as f64,
            (pan_position.1 + pan_delta.1) / *zoom_level as f64,
        );

        cr.paint().expect("Can't paint?");

        Inhibit(false)
    };
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("Project Smokey - Wildfire Prediction");
    window.set_events(EventMask::BUTTON_PRESS_MASK);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(1400, 900);

    // Event box to contain map
    let evt_box = EventBox::builder()
        .events(
            EventMask::BUTTON_PRESS_MASK | EventMask::SCROLL_MASK | EventMask::POINTER_MOTION_MASK,
        )
        .expand(true)
        .has_focus(true)
        .tooltip_text(
            "Click anywhere within the US on this map to get a wildfire probability there",
        )
        .build();

    let map_state = Arc::new(MapState {
        pan_position: RwLock::new((0.0, 0.0)),
        pan_start_pos: RwLock::new((0.0, 0.0)),
        pan_delta: RwLock::new((0.0, 0.0)),
        mouse_position: RwLock::new((0.0, 0.0)),
        zoom_level: RwLock::new(1.0),
        panning: RwLock::new(false),
    });

    // Drawing area to draw map
    let drawing_area = Box::new(DrawingArea::new)();
    drawing_area.connect_draw(draw_map(map_state.clone()));

    evt_box.add(&drawing_area);
    // End drawing area
    // End event box

    // Status bar to display info on where the cursor is
    let statusbar = Arc::new(
        Statusbar::builder()
            .baseline_position(BaselinePosition::Bottom)
            .can_focus(false)
            .halign(Align::Fill)
            .hexpand(true)
            .margin(3)
            .opacity(0.3)
            .height_request(15)
            .build(),
    );
    // End status bar

    let vbox = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();
    vbox.pack_start(&evt_box, true, true, 0);
    vbox.pack_end(statusbar.as_ref(), false, true, 0);

    window.add(&vbox);

    // Set up the window's events, with access to all the widgets and stuff it needs
    event_box_hook_up(&evt_box, statusbar.clone(), map_state.clone());

    window.show_all();
}

fn main() {
    let application = gtk::Application::new(
        Some("com.alexispurslane.project-smokey"),
        Default::default(),
    );

    application.connect_activate(build_ui);

    application.run();
}
