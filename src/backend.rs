use crate::{utils::map::*, MapState};
use std::{sync::Arc, thread, time::Duration};

pub async fn predict((px, py): (f64, f64), map_state: Arc<MapState>) {
    let meters = pixels_to_meters(px, py, &map_state);
    let lonlat = meters_to_lon_lat(meters.0, meters.1);

    thread::sleep(Duration::from_secs(5));
    println!("badoop");
}
