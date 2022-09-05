pub mod map {

    // map image zoom factor: 2.73
    // map image start coordinates: (0, 126)

    use std::sync::Arc;

    use crate::MapState;

    pub const INITIAL_RESOLUTION: f64 = 2.0 * std::f64::consts::PI;
    pub const ORIGIN_SHIFT: f64 = 2.0 * std::f64::consts::PI * 6378137.0 / 2.0;

    pub const BASE_ZOOM_FACTOR: f64 = 33.0234375;
    // the position of the upper left of the map, measured in lat/lon
    pub const MAP_SHIFT_X: f64 = -125.08;
    pub const MAP_SHIFT_Y: f64 = 49.44;

    fn resolution(zoom: f64) -> f64 {
        INITIAL_RESOLUTION / (2.0_f64).powf(zoom)
    }

    pub fn meters_to_lat_lon(mx: f64, my: f64) -> (f64, f64) {
        let lon = (mx / ORIGIN_SHIFT) * 180.0;
        let mut lat = (my / ORIGIN_SHIFT) * 180.0;

        lat = 180.0 / std::f64::consts::PI
            * (2.0 * (lat * std::f64::consts::PI / 180.0).exp().atan()
                - std::f64::consts::PI / 2.0);
        (lat + MAP_SHIFT_X, lon + MAP_SHIFT_Y)
    }

    pub fn pixels_to_meters(px: f64, py: f64, map_state: &Arc<MapState>) -> (f64, f64) {
        println!("pixels to meters:");
        let pp = *map_state.pan_position.read().unwrap();
        let zoom_level = *map_state.zoom_level.read().unwrap() as f64;

        // Adjust for panning and zooming (pixel x,y to absolute x,y)
        let ax = (px - pp.0) / zoom_level;
        let ay = (py - pp.1) / zoom_level;
        let res = self::resolution(zoom_level * BASE_ZOOM_FACTOR);

        let mx = ax * res - ORIGIN_SHIFT;
        let my = ay * res - ORIGIN_SHIFT;

        println!(
            "Click position: {:?}\nPan position: {:?}\nZoom level: {}\nAbsolute position: {:?}\nResolution: {:?}\nMeter position: {:?}",
            (px, py),
            pp,
            zoom_level,
            (ax, ay),
            res,
            (mx, my)
        );

        (mx, my)
    }
}
