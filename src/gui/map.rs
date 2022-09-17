use std::sync::{Arc, RwLock};

use gtk::{
    cairo::Context,
    gdk::{Event, EventMotion, EventScroll, ScrollDirection},
    gdk_pixbuf::Pixbuf,
    prelude::*,
    DrawingArea, EventBox, Statusbar,
};

use crate::utils::map::*;

pub struct MapState {
    pub pan_position: RwLock<(f64, f64)>,
    pub pan_start_pos: RwLock<(f64, f64)>,
    pub pan_delta: RwLock<(f64, f64)>,
    pub panning: RwLock<bool>,

    pub zoom_level: RwLock<f32>,

    pub mouse_position: RwLock<(f64, f64)>,
}

pub fn event_box_hook_up(
    evt_box: &EventBox,
    statusbar: Arc<Statusbar>,
    map_state: Arc<MapState>,
    predict: impl Fn((f64, f64)) -> () + 'static,
) {
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
            if !did_pan {
                let (px, py) = event.position();
                let meters = pixels_to_meters(px, py, &map_state);
                let lonlat = meters_to_lon_lat(meters.0, meters.1);
                println!("click!");
                predict(lonlat)
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

pub fn draw_map(map_state: Arc<MapState>) -> impl Fn(&DrawingArea, &Context) -> Inhibit {
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
