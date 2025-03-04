use crate::{
    helper,
    logic::{
        common_enum, common_model,
        connection_definition::{self, GetPublicConnectionDetailsRequest},
        connection_model_schema, connection_oauth_definition,
        event_access::create_event_access_for_new_user,
        openapi, read, schema_generator, tracker,
    },
    middleware::jwt_auth::{self, JwtState},
    server::AppState,
};
use axum::{
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post},
    Router,
};
use osentities::{
    common_model::{CommonEnum, CommonModel},
    connection_definition::{ConnectionDefinition, PublicConnectionDetails},
    telemetry::log_request_middleware,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub fn get_router(state: &Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .nest(
            "/sdk",
            Router::new()
                .route(
                    "/common-models",
                    get(read::<common_model::CreateRequest, CommonModel>),
                )
                .route(
                    "/common-enums",
                    get(read::<common_enum::GetRequest, CommonEnum>),
                ),
        )
        .nest("/schemas", schema_generator::get_router())
        .nest("/mark", tracker::get_router())
        .route(
            "/connection-data",
            get(read::<GetPublicConnectionDetailsRequest, PublicConnectionDetails>),
        )
        .route(
            "/connection-data/:model/:platform_name",
            get(connection_definition::public_get_connection_details),
        )
        .route(
            "/connection-data/models/:platform_name",
            get(connection_model_schema::get_platform_models),
        )
        .route(
            "/connection-definitions",
            get(read::<connection_definition::CreateRequest, ConnectionDefinition>),
        )
        .route(
            "/connection-oauth-definition-schema",
            get(read::<
                connection_oauth_definition::FrontendOauthConnectionDefinition,
                connection_oauth_definition::FrontendOauthConnectionDefinition,
            >),
        )
        .route(
            "/event-access/default",
            post(create_event_access_for_new_user).layer(from_fn_with_state(
                Arc::new(JwtState::from_state(state)),
                jwt_auth::jwt_auth_middleware,
            )),
        )
        .route("/generate-id/:prefix", get(helper::generate_id))
        .route("/openapi", get(openapi::get_openapi))
        .layer(from_fn(log_request_middleware))
        .layer(TraceLayer::new_for_http())
}
