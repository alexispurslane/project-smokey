use std::{thread, time::Duration};

use gtk::glib::random_double_range;

pub static DANGER_CUTOFF: f64 = 200.0;
pub static WARNING_CUTOFF: f64 = 150.0;

pub struct Prediction(pub f64);

pub fn predict((lon, lat): (f64, f64)) -> Prediction {
    thread::sleep(Duration::from_secs(5));
    Prediction(random_double_range(100.0, 250.0))
}
