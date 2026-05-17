use serde::Serialize;

#[derive(Serialize)]
pub struct ResourceEntry {
    pub resource_type: String,
    pub amount:        f64,
    pub cap:           f64,
    pub rate_per_tick: f64,
}

#[derive(Serialize)]
pub struct ResourcesResponse {
    pub resources: Vec<ResourceEntry>,
    pub tick:      u64,
    pub speed:     u64,
}
