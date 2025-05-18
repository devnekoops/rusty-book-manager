use std::net::{Ipv4Addr, SocketAddr};
use axum::{extract::State, http::StatusCode, routing::get, Router};
use tokio::net::TcpListener;
use anyhow::Result;
use sqlx::{database, postgres::PgConnectOptions, PgPool};

struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String, 
    pub password: String,
    pub database: String,
}

impl From<DatabaseConfig> for PgConnectOptions {
    fn from (cfg: DatabaseConfig) -> Self {
        Self::new()
        .host(&cfg.host)
        .port(cfg.port)
        .username(&cfg.username)
        .password(&cfg.password)
        .database(&cfg.database)
    }
}

fn connect_database_with(cfg: DatabaseConfig) -> PgPool {
    PgPool::connect_lazy_with(cfg.into())
}

// health
pub async fn health_check() -> StatusCode {
    StatusCode::OK
} 

#[tokio::test]
async fn health_check_works() {
    let status_code = health_check().await;
    assert_eq!(status_code, StatusCode::OK);
}

async fn health_check_db(State(db): State<PgPool>) -> StatusCode {
    println!("{:?}", &db);
    let connection_result = sqlx::query("select 1").fetch_one(&db).await;
    match connection_result {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[sqlx::test]
async fn health_check_db_works(pool: sqlx::PgPool) {
    println!("{:?}", &pool);
    let status_code = health_check_db(State(pool)).await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::main]
async fn main() -> Result<()>{

    let database_cfg = DatabaseConfig {
        host: "localhost".into(),
        port: 5432,
        username: "app".into(),
        password: "passwd".into(),
        database: "app".into()
    };

    let conn_pool = connect_database_with(database_cfg);
    
    // router
    let app = Router::new()
    .route("/health", get(health_check))
    .route("/health/db", get(health_check_db))
    .with_state(conn_pool);

    // socket 
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    
    // bind
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on {}", addr);

    // start server
    Ok(axum::serve(listener, app).await?)
}
