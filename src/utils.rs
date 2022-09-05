pub mod map {

    // map image zoom factor: 2.73
    // map image start coordinates: (0, 126)

    pub const TILE_SIZE: f64 = 700.0;
    pub const INITIAL_RESOLUTION: f64 = 2 * std::f64::consts::PI;
    pub const ORIGIN_SHIFT: f64 = 2 * std::f64::consts::PI * 6378137 / 2.0;
    pub const MAP_SHIFT_Y: f64 = 126.0;

    /// Converts XY points from spherical mercator EPSG:900913 to lat/lon in WGS84 Datum
    fn meters_to_lat_lon(mx: f64, my: f64) -> (f64, f64) {
        let mut lon = (mx / ORIGIN_SHIFT) * 180.0;
        let mut lat = (my / ORIGIN_SHIFT) * 180.0;

        lat = 180 / std::f64::consts::PI
            * (2 * (lat * std::f64::consts::PI / 180).exp().atan() - std::f64::consts::PI / 2.0);
        (lat, lon)
    }

    fn pixels_to_meters(px: f64, py: f64) -> (f64, f64) {
        let mx = px * INITIAL_RESOLUTION - ORIGIN_SHIFT;
        let my = (py + MAP_SHIFT_Y) * INITIAL_RESOLUTION - ORIGIN_SHIFT;
        (mx, my)
    }
}
