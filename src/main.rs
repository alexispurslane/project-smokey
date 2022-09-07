use std::sync::{Arc, RwLock};

use gtk::{
    gdk::EventMask,
    glib::{self, translate::ToGlibPtr, MainContext, Receiver, Sender},
    prelude::*,
    Align, BaselinePosition, Dialog, DrawingArea, EventBox, Label, Orientation, Statusbar,
};

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

mod backend;
use crate::backend::predict;

mod gui;
use crate::gui::map::*;

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

    // Set up the window's events, with access to all the widgets and stuff it
    // needs
    let (lonlat_send, lonlat_recv) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let (prediction_send, prediction_recv) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    event_box_hook_up(
        &evt_box,
        statusbar.clone(),
        map_state.clone(),
        lonlat_send.clone(),
    );

    let background_context = MainContext::new();
    std::thread::spawn(move || {
        lonlat_recv.attach(Some(&background_context), move |received| {
            println!(
                "[BACKGROUND THREAD] - Coordinates for prediction received: {:?}",
                received
            );
            let pred = predict(received);
            prediction_send.send(pred).unwrap();
            println!("[BACKGROUND THREAD] - Prediction sent");
            glib::Continue(true)
        });
    });

    window.show_all();

    let dialog = Dialog::builder()
        .title("Prediction Results")
        .transient_for(&window)
        .destroy_with_parent(true)
        .build();
    let ca = dialog.content_area();
    let label = Label::new(Some(
        "<span foreground=\"blue\" size=\"x-large\">Prediction: N/A</span>",
    ));
    ca.pack_start(&label, true, true, 30);
    prediction_recv.attach(None, move |pred| {
        println!("[GUI THREAD] - Prediction received: {}", pred.0);
        let s = format!(
            "<span foreground=\"blue\" size=\"x-large\">Prediction: {}</span>",
            pred.0
        );
        label.set_markup(&s);
        println!("Running dialog...");
        dialog.run();
        glib::Continue(true)
    });
}

fn main() {
    let application = gtk::Application::new(
        Some("com.alexispurslane.project-smokey"),
        Default::default(),
    );

    application.connect_activate(build_ui);

    application.run();
}
