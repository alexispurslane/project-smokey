use std::{error::Error, fmt, path::Path, sync::Mutex};

use gtk::glib::random_double_range;
use tensorflow::{Code, Graph, SavedModelBundle, SessionOptions, SessionRunArgs, Status, Tensor};

#[derive(Debug, Clone)]
struct NoModelError;

impl fmt::Display for NoModelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "no learning model loaded to make prediction with!")
    }
}

impl Error for NoModelError {}

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

pub fn predict((lon, lat): (f64, f64)) -> Result<f64, Box<dyn Error + Send + Sync>> {
    let bundle = BUNDLE.lock().unwrap();
    let bundle = bundle.as_ref().ok_or(NoModelError)?;

    let graph = GRAPH.lock().unwrap();
    let graph = graph.as_ref().ok_or(NoModelError)?;

    let signature = bundle.meta_graph_def().get_signature("serving_default")?;

    let input_info = signature.get_input("model_in")?;
    let output_info = signature.get_output("model_out")?;

    let input_op = graph
        .operation_by_name_required(&input_info.name().name)
        .map_err(|e| Box::new(e))?;
    let output_op = graph
        .operation_by_name_required(&output_info.name().name)
        .map_err(|e| Box::new(e))?;

    let tensor = Tensor::new(&[1, 27]).with_values(&[
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ])?;
    let mut args = SessionRunArgs::new();
    args.add_feed(&input_op, 0, &tensor);

    let out = args.request_fetch(&output_op, 0);

    bundle.session.run(&mut args)?;

    let out = args
        .fetch(out)
        .map(|x: Tensor<f64>| x[0] as f64)
        .map_err(|e| Box::new(e))?;
    Ok(out)
}
