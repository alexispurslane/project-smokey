use crate::backend;

pub mod map {

    use proj::Proj;
    use std::f64::consts::*;
    use std::sync::Arc;

    use crate::MapState;

    // Width (and height, since mercator is square) of the earth when unrolled
    pub const ORIGIN_SHIFT: f64 = 2.0 * PI * (6378137.0 / 2.0);

    // Meters per pixel of the hypothetical extrapolated full world version of
    // the rawmap (8454px x 8454px or so)
    pub const RESOLUTION: f64 = (2.0 * PI * 6378137.0) / 8454.0;

    // the position of the upper left of the map, measured in meters,
    // calculated in the smaller 700x700 Wikipedia reference image, so we have
    // to use a different resolution
    pub const MAP_SHIFT_X: f64 = 108.0 * (2.0 * PI * 6378137.0) / 700.0;
    pub const MAP_SHIFT_Y: f64 = 241.0 * (2.0 * PI * 6378137.0) / 700.0;

    pub fn meters_to_lon_lat(mx: f64, my: f64) -> (f64, f64) {
        // longitude is right to left
        // lattitude is top to bottom
        let from = "EPSG:3857";
        let to = "EPSG:4326";
        let merc_meters_to_lonlat = Proj::new_known_crs(&from, &to, None).unwrap();
        let (lon, lat) = merc_meters_to_lonlat.convert((mx, my)).unwrap();
        (lon, lat)
    }

    pub fn pixels_to_image_coordinates(px: f64, py: f64, app_state: &Arc<MapState>) -> (f64, f64) {
        let pp = *app_state.pan_position.read().unwrap();
        let zoom_level = *app_state.zoom_level.read().unwrap() as f64;

        // Adjust for panning and zooming (pixel x,y to absolute x,y)

        // NOTE: We know (ax, ay) are the exact perfectly accurate
        // pixel-coordinates of the click in the rawmap
        let ax = (px - pp.0) / zoom_level;
        let ay = (py - pp.1) / zoom_level;
        (ax, ay)
    }

    pub fn pixels_to_meters(px: f64, py: f64, app_state: &Arc<MapState>) -> (f64, f64) {
        let (ax, ay) = pixels_to_image_coordinates(px, py, app_state);

        let mx = ax * RESOLUTION + MAP_SHIFT_X;
        let my = ay * RESOLUTION + MAP_SHIFT_Y;
        // The problem with this my is that it's measured from the top left
        // corner of the north pole (in mercator) and that's not how mercator
        // coordinates actually work --- they measure from the equator! So we
        // need to subtract from half the height of the unrolled mercator Earth
        let mx = mx - ORIGIN_SHIFT;
        let my = ORIGIN_SHIFT - my;

        (mx, my)
    }
}

pub fn format_prediction(pred: Result<f64, Box<dyn std::error::Error + Send + Sync>>) -> String {
    match pred {
        Ok(pred) => {
            let color = if pred >= backend::DANGER_CUTOFF {
                "red"
            } else if pred > backend::WARNING_CUTOFF {
                "orange"
            } else {
                "green"
            };
            format!(
                "Prediction: <span color=\"{}\" size=\"x-large\">{:.2}</span>",
                color, pred
            )
        }
        Err(error) => {
            format!(
                "<span color=\"red\" size=\"x-large\">⚠ Error:</span> {:?}",
                error
            )
        }
    }
}
