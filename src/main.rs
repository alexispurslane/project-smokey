use std::{
    borrow::BorrowMut,
    cell::RefCell,
    io::Error,
    sync::{Arc, Mutex, RwLock},
};

use gtk::{
    cairo::{ImageSurface, Surface},
    gdk::{Event, EventMask, EventMotion, EventScroll, ModifierType, ScrollDirection},
    gdk_pixbuf::{InterpType, Pixbuf},
    glib::clone,
    prelude::*,
    DrawingArea, EventBox, Image,
};

struct MapState {
    pan_position: RwLock<(f64, f64)>,
    panning: RwLock<bool>,
    zoom_level: RwLock<f32>,
    mouse_position: RwLock<(f64, f64)>,
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("Project Smokey - Wildfire Prediction");
    window.set_events(EventMask::BUTTON_PRESS_MASK);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(700, 700);

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
        mouse_position: RwLock::new((0.0, 0.0)),
        zoom_level: RwLock::new(1.0),
        panning: RwLock::new(false),
    });

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

        evt_box.connect_button_press_event(move |evt_box, event| {
            let mut panning = map_state.panning.write().unwrap();
            *panning = true;

            Inhibit(false)
        });
    }

    {
        let map_state = map_state.clone();

        evt_box.connect_button_release_event(move |evt_box, event| {
            let mut panning = map_state.panning.write().unwrap();
            *panning = false;

            Inhibit(false)
        });
    }

    {
        let map_state = map_state.clone();

        evt_box.connect_motion_notify_event(move |evt_box, motion_event: &EventMotion| {
            let mut mouse_position = map_state.mouse_position.write().unwrap();
            *mouse_position = motion_event.position();
            println!("Mouse position: {:?}", mouse_position);

            if *map_state.panning.read().unwrap() {
                let mut pan_position = map_state.pan_position.write().unwrap();
                *pan_position = motion_event.position();
                println!("Pan position: {:?}", pan_position);

                evt_box.child().unwrap().queue_draw();
            }
            Inhibit(false)
        });
    }

    let pixbuf = Pixbuf::from_file("mercator-projected-map.jpg")
        .expect("Can't load image necessary for application");
    let drawing_area = Box::new(DrawingArea::new)();
    drawing_area.connect_draw(move |_, cr| {
        let zoom_level = map_state.zoom_level.read().unwrap();
        cr.scale(*zoom_level as f64, *zoom_level as f64);

        let pan_position = map_state.pan_position.read().unwrap();
        cr.set_source_pixbuf(
            &pixbuf,
            (pan_position.0) / *zoom_level as f64 - 130.0,
            (pan_position.1) / *zoom_level as f64 - 94.0,
        );

        cr.paint();

        Inhibit(false)
    });
    evt_box.add(&drawing_area);

    window.add(&evt_box);

    window.show_all();
}

fn main() {
    let application =
        gtk::Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default());

    application.connect_activate(build_ui);

    application.run();
}
