use axum::{Json, extract::State, response::IntoResponse};
use pengu::bot::callback_payload::{CallbackData, CallbackPayload, validation::ValidationResponse};
use tracing::{debug, instrument};

use crate::primary::state::AppState;

#[instrument(skip_all, level = "debug", name = "qqbot-callback")]
pub async fn handler(
    State(state): State<AppState>,
    Json(payload): Json<CallbackPayload>,
) -> axum::response::Response {
    debug!(
        message = "Handling callback",
        operation = ?payload.op_code,
        event_id = ?payload.event_id,
        event_type = ?payload.event_type
    );

    if !payload.valid() {
        debug!(message = "Reject because payload is invalid");
        return http::StatusCode::BAD_REQUEST.into_response();
    }

    let CallbackPayload { data, .. } = payload;

    match data {
        CallbackData::CallbackValidation(date) => {
            let signature = state
                .qqbot
                .compute_signature(&date.bytes_iter().collect::<Vec<_>>());
            debug!(message = "Computed signature", signature);

            Json(ValidationResponse {
                plain_token: date.plain_token,
                signature,
            })
            .into_response()
        }

        _ => http::StatusCode::NOT_IMPLEMENTED.into_response(),
    }
}
