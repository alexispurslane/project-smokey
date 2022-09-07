use gtk::glib::random_double_range;

use std::{thread, time::Duration};

pub struct Prediction(pub f64);

pub fn predict((lon, lat): (f64, f64)) -> Prediction {
    thread::sleep(Duration::from_secs(5));
    Prediction(random_double_range(0.0, 1.0))
}
