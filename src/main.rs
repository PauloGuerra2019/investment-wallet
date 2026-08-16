use argon2::{Argon2, PasswordHash, PasswordVerifier};
use argon2::password_hash::{PasswordHasher, SaltString};
use axum::{extract::{Form, State}, http::{header, HeaderMap, StatusCode}, response::{Html, IntoResponse, Redirect, Response}, routing::{get, post}, Router};
use askama::Template;
use password_hash::rand_core::OsRng;
use serde::Deserialize;
use sqlx::{PgPool, Row};
use std::{env, net::SocketAddr};
use tower_http::{services::ServeDir, trace::TraceLayer};
use uuid::Uuid;

#[derive(Clone)]
struct AppState { db: PgPool }

#[derive(Template)]
#[template(source="<!doctype html><html lang='pt-BR'><head><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>Carteira — Login</title><link rel='stylesheet' href='/static/app.css'></head><body><main><h1>Carteira de Investimentos</h1><h2>Entrar</h2>{% if error.is_some() %}<p class='error'>{{ error.as_ref().unwrap() }}</p>{% endif %}<form method='post' action='/login'><input name='email' type='email' placeholder='E-mail' required><input name='password' type='password' placeholder='Senha' minlength='8' required><button>Entrar</button></form><p>Não possui conta? <a href='/register'>Cadastre-se</a></p></main></body></html>", ext="html")]
struct LoginPage { error: Option<String> }

#[derive(Template)]
#[template(source="<!doctype html><html lang='pt-BR'><head><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>Carteira — Cadastro</title><link rel='stylesheet' href='/static/app.css'></head><body><main><h1>Carteira de Investimentos</h1><h2>Criar conta</h2>{% if error.is_some() %}<p class='error'>{{ error.as_ref().unwrap() }}</p>{% endif %}<form method='post' action='/register'><input name='email' type='email' placeholder='E-mail' required><input name='password' type='password' placeholder='Senha (mín. 8 caracteres)' minlength='8' required><button>Criar conta</button></form><p>Já possui conta? <a href='/login'>Entrar</a></p></main></body></html>", ext="html")]
struct RegisterPage { error: Option<String> }

#[derive(Template)]
#[template(source="<!doctype html><html lang='pt-BR'><head><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>Carteira Rust</title><link rel='stylesheet' href='/static/app.css'></head><body><main><nav><strong>Carteira</strong><form method='post' action='/logout'><button>Sair</button></form></nav><h1>Dashboard</h1><p>{{ email }}</p><div class='cards'><section><small>Investido</small><strong>R$ {{ total }}</strong></section><section><small>Ativos</small><strong>{{ assets }}</strong></section></div><h2>Movimentações</h2><table><tr><th>Ativo</th><th>Tipo</th><th>Qtd.</th><th>Preço</th></tr>{% for x in transactions %}<tr><td>{{ x.symbol }}</td><td>{{ x.kind }}</td><td>{{ x.quantity }}</td><td>R$ {{ x.price }}</td></tr>{% endfor %}</table><form method='post' action='/transactions'><input name='symbol' placeholder='PETR4' required><select name='kind'><option>BUY</option><option>SELL</option></select><input name='quantity' type='number' min='0.0001' step='0.0001' placeholder='Quantidade' required><input name='price' type='number' min='0.01' step='0.01' placeholder='Preço' required><button>Registrar</button></form></main></body></html>", ext="html")]
struct Dashboard { email: String, total: String, assets: i64, transactions: Vec<TxView> }
struct TxView { symbol: String, kind: String, quantity: String, price: String }

#[derive(Deserialize)]
struct Credentials { email: String, password: String }

#[derive(Deserialize)]
struct TransactionForm { symbol: String, kind: String, quantity: rust_decimal::Decimal, price: rust_decimal::Decimal }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().with_env_filter(env::var("RUST_LOG").unwrap_or_else(|_| "info".into())).init();
    let db = PgPool::connect(&env::var("DATABASE_URL")?).await?;
    sqlx::migrate!().run(&db).await?;

    let app = Router::new()
        .route("/", get(root))
        .route("/login", get(login_page).post(login))
        .route("/register", get(register_page).post(register))
        .route("/logout", post(logout))
        .route("/transactions", post(create_transaction))
        .route("/health", get(|| async { "ok" }))
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState { db });

    let addr: SocketAddr = "0.0.0.0:3000".parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "investment wallet listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn root(headers: HeaderMap, State(state): State<AppState>) -> Response {
    if current_user(&headers, &state.db).await.is_some() { dashboard(headers, State(state)).await.into_response() } else { Redirect::to("/login").into_response() }
}

async fn login_page() -> impl IntoResponse { Html(LoginPage { error: None }.render().unwrap()) }
async fn register_page() -> impl IntoResponse { Html(RegisterPage { error: None }.render().unwrap()) }

async fn register(State(state): State<AppState>, Form(form): Form<Credentials>) -> Response {
    let email = form.email.trim().to_lowercase();
    if form.password.len() < 8 { return Html(RegisterPage { error: Some("A senha deve ter pelo menos 8 caracteres.".into()) }.render().unwrap()).into_response(); }
    let salt = SaltString::generate(&mut OsRng);
    let hash = match Argon2::default().hash_password(form.password.as_bytes(), &salt) { Ok(v) => v.to_string(), Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response() };
    let user_id = Uuid::new_v4();
    let result = sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1,$2,$3)").bind(user_id).bind(&email).bind(hash).execute(&state.db).await;
    if result.is_err() { return Html(RegisterPage { error: Some("E-mail já cadastrado ou inválido.".into()) }.render().unwrap()).into_response(); }
    create_session_response(&state.db, user_id).await
}

async fn login(State(state): State<AppState>, Form(form): Form<Credentials>) -> Response {
    let email = form.email.trim().to_lowercase();
    let row = sqlx::query("SELECT id, password_hash FROM users WHERE email = $1").bind(&email).fetch_optional(&state.db).await.ok().flatten();
    let Some(row) = row else { return Html(LoginPage { error: Some("E-mail ou senha inválidos.".into()) }.render().unwrap()).into_response(); };
    let user_id: Uuid = row.get("id");
    let hash: String = row.get("password_hash");
    let valid = PasswordHash::new(&hash).ok().and_then(|p| Argon2::default().verify_password(form.password.as_bytes(), &p).ok()).is_some();
    if !valid { return Html(LoginPage { error: Some("E-mail ou senha inválidos.".into()) }.render().unwrap()).into_response(); }
    create_session_response(&state.db, user_id).await
}

async fn create_session_response(db: &PgPool, user_id: Uuid) -> Response {
    let session = Uuid::new_v4();
    if sqlx::query("INSERT INTO sessions (id, user_id, expires_at) VALUES ($1,$2,now() + interval '7 days')").bind(session).bind(user_id).execute(db).await.is_err() { return StatusCode::INTERNAL_SERVER_ERROR.into_response(); }
    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, format!("session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=604800", session).parse().unwrap());
    (headers, Redirect::to("/")).into_response()
}

async fn logout(headers: HeaderMap, State(state): State<AppState>) -> Response {
    if let Some(session) = session_id(&headers) { let _ = sqlx::query("DELETE FROM sessions WHERE id = $1").bind(session).execute(&state.db).await; }
    let mut response = Redirect::to("/login").into_response();
    response.headers_mut().insert(header::SET_COOKIE, "session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0".parse().unwrap());
    response
}

async fn current_user(headers: &HeaderMap, db: &PgPool) -> Option<(Uuid, String)> {
    let session = session_id(headers)?;
    sqlx::query("SELECT u.id, u.email FROM sessions s JOIN users u ON u.id = s.user_id WHERE s.id = $1 AND s.expires_at > now()").bind(session).fetch_optional(db).await.ok().flatten().map(|r| (r.get("id"), r.get("email")))
}

fn session_id(headers: &HeaderMap) -> Option<Uuid> {
    headers.get(header::COOKIE)?.to_str().ok()?.split(';').find_map(|p| { let mut it = p.trim().splitn(2, '='); if it.next()? == "session" { Uuid::parse_str(it.next()?).ok() } else { None } })
}

async fn dashboard(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
    let Some((user_id, email)) = current_user(&headers, &state.db).await else { return Redirect::to("/login").into_response(); };
    let total: rust_decimal::Decimal = sqlx::query_scalar("SELECT COALESCE(SUM(quantity * price), 0) FROM transactions WHERE user_id = $1 AND kind = 'BUY'").bind(user_id).fetch_one(&state.db).await.unwrap_or_default();
    let assets: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT symbol) FROM transactions WHERE user_id = $1").bind(user_id).fetch_one(&state.db).await.unwrap_or(0);
    let rows = sqlx::query("SELECT symbol, kind, quantity, price FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50").bind(user_id).fetch_all(&state.db).await.unwrap_or_default();
    let transactions = rows.into_iter().map(|r| TxView { symbol: r.get("symbol"), kind: r.get("kind"), quantity: r.get::<rust_decimal::Decimal,_>("quantity").to_string(), price: r.get::<rust_decimal::Decimal,_>("price").to_string() }).collect();
    Html(Dashboard { email, total: total.to_string(), assets, transactions }.render().unwrap()).into_response()
}

async fn create_transaction(headers: HeaderMap, State(state): State<AppState>, Form(form): Form<TransactionForm>) -> Response {
    let Some((user_id, _)) = current_user(&headers, &state.db).await else { return Redirect::to("/login").into_response(); };
    if form.quantity <= rust_decimal::Decimal::ZERO || form.price <= rust_decimal::Decimal::ZERO || !matches!(form.kind.as_str(), "BUY" | "SELL") { return StatusCode::BAD_REQUEST.into_response(); }
    let _ = sqlx::query("INSERT INTO transactions (id, user_id, symbol, kind, quantity, price) VALUES ($1,$2,$3,$4,$5,$6)").bind(Uuid::new_v4()).bind(user_id).bind(form.symbol.trim().to_uppercase()).bind(form.kind).bind(form.quantity).bind(form.price).execute(&state.db).await;
    Redirect::to("/").into_response()
}
