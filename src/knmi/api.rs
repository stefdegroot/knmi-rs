use axum::{Router, routing::get};

use crate::{AppState, knmi, knmi::sources::KnmiSource};

pub mod forecast;

pub trait Api {
    fn set_route (&self, app: Router, state: AppState) -> Router;
}

impl Api for KnmiSource {
    fn set_route (&self, app: Router, state: AppState) -> Router {
        app.route(
            &format!("/{}/{}", self.id, self.version),
            get(knmi::api::forecast::forecast).with_state(state)
        )
    }
}