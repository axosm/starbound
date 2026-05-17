use crate::auth::handlers as auth;
use crate::handlers::{admin, alliance, fleet, game, planet, research, resources, space, speed, sse_handler, units};
use crate::state::AppState;
use axum::{routing::{delete, get, post, put}, Router};
use std::sync::Arc;
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};

pub fn build_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            state.cfg.cors_origin
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // ── Auth ──────────────────────────────────────────────
        .route("/api/auth/register",            post(auth::register))
        .route("/api/auth/login",               post(auth::login))

        // ── Game init ─────────────────────────────────────────
        .route("/api/game/init",                get(game::game_init))

        // ── Universe / map ────────────────────────────────────────
        .route("/api/systems",                  get(game::list_systems))
        .route("/api/systems/:id/planets",      get(game::list_system_planets))

        // ── Speed ─────────────────────────────────────────────
        .route("/api/speed",                    get(speed::get_speed))
        .route("/api/speed",                    post(speed::set_speed))

        // ── Planets ───────────────────────────────────────────
        .route("/api/planets/:id",              get(planet::get_planet))
        .route("/api/planets/:id/build",        post(planet::build_on_tile))

        // ── Units / fleet ─────────────────────────────────────
        .route("/api/units",                    get(fleet::list_units))
        .route("/api/units/move",               post(fleet::move_unit))
        .route("/api/units/defs",               get(units::unit_defs))
        .route("/api/units/recruit",            post(units::recruit))

        // ── Space ops ─────────────────────────────────────────
        .route("/api/space/scan",               post(space::scan))
        .route("/api/space/cloak",              post(space::set_cloak))
        .route("/api/space/reports",            get(space::battle_reports))
        .route("/api/space/reports/read",       post(space::mark_reports_read))

        // ── Research ──────────────────────────────────────────
        .route("/api/research",                 get(research::list_research))
        .route("/api/research/start",           post(research::start_research))

        // ── Resources ─────────────────────────────────────────
        .route("/api/resources",                get(resources::get_resources))

        // ── Empires / alliances ───────────────────────────────
        .route("/api/empire",                   get(alliance::get_my_empire))
        .route("/api/empire",                   post(alliance::create_empire))
        .route("/api/empire/leave",             post(alliance::leave))
        .route("/api/empire/:id/invite",        post(alliance::invite))
        .route("/api/empire/:id/kick/:pid",     delete(alliance::kick))
        .route("/api/empire/:id/role",          put(alliance::set_role))

        // ── SSE (battle alerts, tick sync) ────────────────────
        .route("/api/events",                   get(sse_handler::sse_handler))
        .route("/api/events/missed",            get(sse_handler::missed_events))

        // ── Admin ─────────────────────────────────────────────
        .route("/api/admin/status",             get(admin::server_status))
        .route("/api/admin/planets/:id/generate", post(admin::generate_planet_tiles))

        // ── Static frontend ───────────────────────────────────
        .fallback_service(tower_http::services::ServeDir::new("../frontend/dist"))

        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
