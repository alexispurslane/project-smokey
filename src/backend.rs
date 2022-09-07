use crate::{utils::map::*, MapState};
use std::{
    sync::{mpsc::Sender, Arc},
    thread,
    time::Duration,
};

pub async fn predict((lon, lat): (f64, f64), send_result: Sender<f64>) {
    thread::sleep(Duration::from_secs(5));
    send_result.send(0.0).unwrap();
}
