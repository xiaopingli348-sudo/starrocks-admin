use axum::{
    middleware as axum_middleware,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod db;
mod handlers;
mod middleware;
mod models;
mod services;
mod utils;

use config::Config;
use services::{
    AuthService, ClusterService, MetricsCollectorService, MySQLPoolManager, 
    OverviewService, SystemFunctionService,
};
use sqlx::SqlitePool;
use utils::JwtUtil;

// Application state
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub mysql_pool_manager: MySQLPoolManager,
    pub auth_service: AuthService,
    pub cluster_service: ClusterService,
    pub system_function_service: SystemFunctionService,
    pub metrics_collector_service: MetricsCollectorService,
    pub overview_service: OverviewService,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth::register,
        handlers::auth::login,
        handlers::auth::get_me,
        handlers::cluster::create_cluster,
        handlers::cluster::list_clusters,
        handlers::cluster::get_cluster,
        handlers::cluster::update_cluster,
        handlers::cluster::delete_cluster,
        handlers::cluster::get_cluster_health,
        handlers::backend::list_backends,
        handlers::frontend::list_frontends,
        handlers::materialized_view::list_materialized_views,
        handlers::materialized_view::get_materialized_view,
        handlers::materialized_view::get_materialized_view_ddl,
        handlers::materialized_view::create_materialized_view,
        handlers::materialized_view::delete_materialized_view,
        handlers::materialized_view::refresh_materialized_view,
        handlers::materialized_view::cancel_refresh_materialized_view,
        handlers::materialized_view::alter_materialized_view,
        handlers::query::list_queries,
        handlers::query::kill_query,
        handlers::query::execute_sql,
        handlers::query_history::list_query_history,
        handlers::sessions::get_sessions,
        handlers::sessions::kill_session,
        handlers::variables::get_variables,
        handlers::variables::update_variable,
        handlers::profile::list_profiles,
        handlers::profile::get_profile,
        handlers::query_profile::get_query_profile,
        handlers::system_management::get_system_functions,
        handlers::system_management::get_system_function_detail,
        handlers::system::get_runtime_info,
        handlers::monitor::get_metrics_summary,
        handlers::overview::get_cluster_overview,
        handlers::overview::get_health_cards,
        handlers::overview::get_performance_trends,
        handlers::overview::get_resource_trends,
    ),
    components(
        schemas(
            models::User,
            models::UserResponse,
            models::CreateUserRequest,
            models::LoginRequest,
            models::LoginResponse,
            models::Cluster,
            models::ClusterResponse,
            models::CreateClusterRequest,
            models::UpdateClusterRequest,
            models::ClusterHealth,
            models::HealthStatus,
            models::HealthCheck,
            models::Backend,
            models::Frontend,
            models::MaterializedView,
            models::CreateMaterializedViewRequest,
            models::RefreshMaterializedViewRequest,
            models::AlterMaterializedViewRequest,
            models::MaterializedViewDDL,
            models::Query,
            models::QueryExecuteRequest,
            models::QueryExecuteResponse,
            models::QueryHistoryItem,
            models::QueryHistoryResponse,
            models::ProfileListItem,
            models::ProfileDetail,
            models::RuntimeInfo,
            models::MetricsSummary,
            models::SystemFunction,
            models::CreateFunctionRequest,
            models::UpdateOrderRequest,
            models::FunctionOrder,
            services::ClusterOverview,
            services::HealthCard,
            services::HealthStatus,
            services::PerformanceTrends,
            services::ResourceTrends,
            services::MetricsSnapshot,
        )
    ),
    tags(
        (name = "Authentication", description = "User authentication endpoints"),
        (name = "Clusters", description = "Cluster management endpoints"),
        (name = "Backends", description = "Backend node management"),
        (name = "Frontends", description = "Frontend node management"),
        (name = "Materialized Views", description = "Materialized view management"),
        (name = "Queries", description = "Query management"),
        (name = "Profiles", description = "Query profile management"),
        (name = "System", description = "System information"),
        (name = "Monitoring", description = "Monitoring and metrics"),
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            utoipa::openapi::security::SecurityScheme::Http(
                utoipa::openapi::security::Http::new(
                    utoipa::openapi::security::HttpAuthScheme::Bearer,
                ),
            ),
        );
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration first
    let config = Config::load()?;
    
    // Initialize logging
    let log_filter = tracing_subscriber::EnvFilter::new(&config.logging.level);
    
    let registry = tracing_subscriber::registry().with(log_filter);
    
    // Add file logging if configured
    if let Some(log_file) = &config.logging.file {
        // Ensure log directory exists
        if let Some(parent) = std::path::Path::new(log_file).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        
        let file_appender = tracing_appender::rolling::daily("logs", "starrocks-admin.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        registry
            .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
            .with(tracing_subscriber::fmt::layer())
            .init();
    } else {
        registry
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
    tracing::info!("StarRocks Admin starting up");
    tracing::info!("Configuration loaded successfully");

    let pool = db::create_pool(&config.database.url).await?;
    tracing::info!("Database pool created successfully");

    let jwt_util = Arc::new(JwtUtil::new(&config.auth.jwt_secret, &config.auth.jwt_expires_in));
    let auth_service = Arc::new(AuthService::new(pool.clone(), (*jwt_util).clone()));
    let cluster_service = Arc::new(ClusterService::new(pool.clone()));
    let mysql_pool_manager = Arc::new(MySQLPoolManager::new());
    let system_function_service = Arc::new(SystemFunctionService::new(
        Arc::new(pool.clone()),
        (*mysql_pool_manager).clone(),
        (*cluster_service).clone(),
    ));
    
    // Create new services for cluster overview
    let metrics_collector_service = Arc::new(MetricsCollectorService::new(
        pool.clone(),
        cluster_service.clone(),
    ));
    let overview_service = Arc::new(OverviewService::new(
        pool.clone(),
        cluster_service.clone(),
    ));

    let app_state = AppState {
        db: pool.clone(),
        mysql_pool_manager: (*mysql_pool_manager).clone(),
        auth_service: (*auth_service).clone(),
        cluster_service: (*cluster_service).clone(),
        system_function_service: (*system_function_service).clone(),
        metrics_collector_service: (*metrics_collector_service).clone(),
        overview_service: (*overview_service).clone(),
    };
    
    // Start metrics collector background task
    let collector_clone = metrics_collector_service.clone();
    tokio::spawn(async move {
        tracing::info!("Starting metrics collector background task");
        collector_clone.start_collection().await;
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false);

    let auth_state = middleware::AuthState {
        jwt_util: jwt_util.clone(),
    };

    let public_routes = Router::new()
        .route("/api/auth/register", post(handlers::auth::register))
        .route("/api/auth/login", post(handlers::auth::login))
        .with_state(auth_service.clone());

    // Routes using ClusterService
    let cluster_routes = Router::new()
        .route("/api/clusters", post(handlers::cluster::create_cluster))
        .route("/api/clusters", get(handlers::cluster::list_clusters))
        .route("/api/clusters/:id", get(handlers::cluster::get_cluster))
        .route("/api/clusters/:id", put(handlers::cluster::update_cluster))
        .route("/api/clusters/:id", delete(handlers::cluster::delete_cluster))
        .route("/api/clusters/:id/backends", get(handlers::backend::list_backends))
        .route("/api/clusters/:id/backends/:host/:port", delete(handlers::backend::delete_backend))
        .route("/api/clusters/:id/frontends", get(handlers::frontend::list_frontends))
        .route("/api/clusters/:id/queries", get(handlers::query::list_queries))
        .route("/api/clusters/:id/system/runtime_info", get(handlers::system::get_runtime_info))
        .route("/api/clusters/:id/metrics/summary", get(handlers::monitor::get_metrics_summary))
        .with_state(cluster_service.clone());
    
    // Routes using OverviewService
    let overview_routes = Router::new()
        .route("/api/clusters/:id/overview", get(handlers::overview::get_cluster_overview))
        .route("/api/clusters/:id/overview/health", get(handlers::overview::get_health_cards))
        .route("/api/clusters/:id/overview/performance", get(handlers::overview::get_performance_trends))
        .route("/api/clusters/:id/overview/resources", get(handlers::overview::get_resource_trends))
        .with_state(overview_service.clone());

    // Routes using AppState  
    let app_state_arc = Arc::new(app_state.clone());
    let app_routes = Router::new()
        .route("/api/clusters/:id/health", get(handlers::cluster::get_cluster_health).post(handlers::cluster::get_cluster_health))
        .route("/api/clusters/:id/materialized_views", get(handlers::materialized_view::list_materialized_views).post(handlers::materialized_view::create_materialized_view))
        .route("/api/clusters/:id/materialized_views/:mv_name", get(handlers::materialized_view::get_materialized_view).delete(handlers::materialized_view::delete_materialized_view).put(handlers::materialized_view::alter_materialized_view))
        .route("/api/clusters/:id/materialized_views/:mv_name/ddl", get(handlers::materialized_view::get_materialized_view_ddl))
        .route("/api/clusters/:id/materialized_views/:mv_name/refresh", post(handlers::materialized_view::refresh_materialized_view))
        .route("/api/clusters/:id/materialized_views/:mv_name/cancel", post(handlers::materialized_view::cancel_refresh_materialized_view))
        .route("/api/clusters/:cluster_id/queries/execute", post(handlers::query::execute_sql))
        .route("/api/clusters/:cluster_id/queries/:query_id", delete(handlers::query::kill_query))
        .route("/api/clusters/:cluster_id/queries/history", get(handlers::query_history::list_query_history))
        .route("/api/clusters/:cluster_id/profiles", get(handlers::profile::list_profiles))
        .route("/api/clusters/:cluster_id/profiles/:query_id", get(handlers::profile::get_profile))
        .route("/api/clusters/:cluster_id/sessions", get(handlers::sessions::get_sessions))
        .route("/api/clusters/:cluster_id/sessions/:session_id", delete(handlers::sessions::kill_session))
        .route("/api/clusters/:cluster_id/variables", get(handlers::variables::get_variables))
        .route("/api/clusters/:cluster_id/variables/:variable_name", put(handlers::variables::update_variable))
        .route("/api/clusters/:cluster_id/queries/:query_id/profile", get(handlers::query_profile::get_query_profile))
        .route("/api/clusters/:cluster_id/system", get(handlers::system_management::get_system_functions))
        .route("/api/clusters/:cluster_id/system/:function_name", get(handlers::system_management::get_system_function_detail))
        .route("/api/clusters/:id/system-functions", get(handlers::system_function::get_system_functions).post(handlers::system_function::create_system_function))
        .route("/api/clusters/:id/system-functions/orders", put(handlers::system_function::update_function_orders))
        .route("/api/clusters/:id/system-functions/:function_id/execute", post(handlers::system_function::execute_system_function))
        .route("/api/clusters/:id/system-functions/:function_id/favorite", put(handlers::system_function::toggle_function_favorite))
        .route("/api/clusters/:id/system-functions/:function_id", put(handlers::system_function::update_function).delete(handlers::system_function::delete_system_function))
        .route("/api/system-functions/:function_name/access-time", put(handlers::system_function::update_system_function_access_time))
        .route("/api/system-functions/category/:category_name", delete(handlers::system_function::delete_category))
        .with_state(app_state_arc);

    // Auth route
    let auth_routes = Router::new()
        .route("/api/auth/me", get(handlers::auth::get_me))
        .with_state(auth_service.clone());

    let protected_routes = Router::new()
        .merge(auth_routes)
        .merge(cluster_routes)
        .merge(overview_routes)
        .merge(app_routes)
        .layer(axum_middleware::from_fn_with_state(
            auth_state,
            middleware::auth_middleware,
        ));

    let health_routes = Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(ready_check));

    // Static file serving (if enabled)
    let static_routes = if config.static_config.enabled {
        tracing::info!("Static file serving enabled, serving from: {}", config.static_config.web_root);
        Router::new()
            .nest_service("/", ServeDir::new(&config.static_config.web_root))
    } else {
        Router::new()
    };

    let app = Router::new()
        .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(public_routes)
        .merge(protected_routes)
        .merge(health_routes)
        .merge(static_routes)
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("API documentation available at http://{}/api-docs", addr);
    tracing::info!("StarRocks Admin is ready to serve requests");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn ready_check() -> &'static str {
    "READY"
}
