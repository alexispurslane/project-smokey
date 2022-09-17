use std::{error::Error, path::Path, sync::Mutex};

use gtk::glib::random_double_range;
use tensorflow::{Code, Graph, SavedModelBundle, SessionOptions, Status};

pub static DANGER_CUTOFF: f64 = 200.0;
pub static WARNING_CUTOFF: f64 = 150.0;

static BUNDLE: Mutex<Option<SavedModelBundle>> = Mutex::new(None);
static GRAPH: Mutex<Option<Graph>> = Mutex::new(None);

pub fn load_model(location: String) -> Result<(), Box<dyn Error>> {
    if !Path::new(&location).exists() {
        return Err(Box::new(
            Status::new_set(
                Code::NotFound,
                &format!(
                    "No such directory '{}', please create a saved model there.",
                    location
                ),
            )
            .unwrap(),
        ));
    }

    let mut graph = Graph::new();
    let bundle = SavedModelBundle::load(&SessionOptions::new(), &["serve"], &mut graph, location)?;
    *GRAPH.lock().unwrap() = Some(graph);
    *BUNDLE.lock().unwrap() = Some(bundle);
    return Ok(());
}

pub fn predict((lon, lat): (f64, f64)) -> Result<f64, &'static str> {
    if BUNDLE.lock().unwrap().is_none() || GRAPH.lock().unwrap().is_none() {
        return Err("No model initialized to run prediction on!");
    }
    Ok(random_double_range(100.0, 250.0))
}
