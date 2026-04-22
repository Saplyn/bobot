use axum::{Json, extract::State, response::IntoResponse};
use pengu::bot::callback_payload::{CallbackData, CallbackPayload, validation::ValidationResponse};

use crate::primary::state::AppState;

pub async fn handler(
    State(state): State<AppState>,
    Json(payload): Json<CallbackPayload>,
) -> axum::response::Response {
    if !payload.valid() {
        return http::StatusCode::BAD_REQUEST.into_response();
    }

    let CallbackPayload { data, .. } = payload;

    match data {
        CallbackData::CallbackValidation(date) => {
            let signature = state
                .qqbot
                .compute_signature(&date.bytes_iter().collect::<Vec<_>>());
            Json(ValidationResponse {
                plain_token: date.plain_token,
                signature,
            })
            .into_response()
        }

        _ => http::StatusCode::NOT_IMPLEMENTED.into_response(),
    }
}
