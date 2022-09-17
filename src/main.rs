use std::{
    sync::{Arc, RwLock},
    thread,
};

use gtk::{
    gdk::EventMask,
    glib::{self, clone, translate::ToGlibPtr},
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
    let dialog = Dialog::builder()
        .title("Prediction Results")
        .transient_for(&window)
        .destroy_with_parent(true)
        .width_request(300)
        .height_request(250)
        .build();
    let ca = dialog.content_area();
    let label = Label::new(Some("Prediction: N/A"));
    ca.pack_start(&label, true, true, 30);
    event_box_hook_up(
        &evt_box,
        statusbar.clone(),
        map_state.clone(),
        move |lonlat| {
            let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
            thread::spawn(move || {
                let pred = backend::predict(lonlat);
                sender.send(pred).unwrap();
            });
            receiver.attach(
                None,
                clone!(@weak label, @weak dialog => @default-return Continue(false), move |pred| {
                    let color = if pred.0 >= backend::DANGER_CUTOFF {
                        "red"
                    } else if pred.0 > backend::WARNING_CUTOFF {
                        "orange"
                    } else {
                        "green"
                    };
                    let s = format!(
                        "Prediction: <span color=\"{}\" size=\"x-large\">{:.2}</span>",
                        color, pred.0
                    );
                    label.set_markup(&s);
                    println!("Running dialog...");
                    dialog.show_all();
                    Continue(true)
                }),
            );
        },
    );

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
