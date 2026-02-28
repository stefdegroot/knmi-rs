use axum::{
    extract::{Json, Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use crate::AppState;

#[derive(Serialize, Deserialize, Debug)]
struct Coordinates {
    lat: f64,
    lon: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForecastInput {
    coords: Coordinates,
}

pub async fn forecast (
    State(state): State<AppState>,
    Json(payload): Json<ForecastInput>,
) -> Response {
    "forecast".into_response()
}
