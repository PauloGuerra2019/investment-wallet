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
#[template(source="<!doctype html><html lang='pt-BR'><head><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>Entrar | Atlas</title><link rel='stylesheet' href='/static/app.css'></head><body class='auth-page'><main class='auth-shell'><section class='auth-aside'><a class='brand' href='/'><span class='brand-mark'>A</span><span>atlas<span class='brand-dot'>.</span></span></a><div class='aside-copy'><p class='eyebrow'>Clareza para investir melhor</p><h1>Sua carteira,<br><em>no seu ritmo.</em></h1><p>Acompanhe cada movimento e mantenha uma visão limpa do seu patrimônio.</p></div><div class='aside-footer'><span class='status-dot'></span> Seus dados ficam protegidos</div></section><section class='auth-content'><div class='auth-card'><p class='eyebrow'>Bem-vindo de volta</p><h2>Entrar na sua conta</h2><p class='muted'>Acesse sua carteira e continue de onde parou.</p>{% if error.is_some() %}<p class='error'>{{ error.as_ref().unwrap() }}</p>{% endif %}<form class='stack-form' method='post' action='/login'><label>E-mail<input name='email' type='email' placeholder='voce@exemplo.com' autocomplete='email' required></label><label>Senha<input name='password' type='password' placeholder='Sua senha' autocomplete='current-password' minlength='8' required></label><button class='button button-primary' type='submit'>Entrar <span aria-hidden='true'>→</span></button></form><p class='auth-switch'>Ainda não possui conta? <a href='/register'>Criar uma conta</a></p></div></section></main></body></html>", ext="html")]
struct LoginPage { error: Option<String> }

#[derive(Template)]
#[template(source="<!doctype html><html lang='pt-BR'><head><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>Criar conta | Atlas</title><link rel='stylesheet' href='/static/app.css'></head><body class='auth-page'><main class='auth-shell'><section class='auth-aside'><a class='brand' href='/'><span class='brand-mark'>A</span><span>atlas<span class='brand-dot'>.</span></span></a><div class='aside-copy'><p class='eyebrow'>Comece com intenção</p><h1>Um lugar para<br><em>ver o todo.</em></h1><p>Registre suas operações, acompanhe seus ativos e tome decisões com mais contexto.</p></div><div class='aside-footer'><span class='status-dot'></span> Configuração simples e segura</div></section><section class='auth-content'><div class='auth-card'><p class='eyebrow'>Primeiro passo</p><h2>Criar sua conta</h2><p class='muted'>Leva menos de um minuto para começar.</p>{% if error.is_some() %}<p class='error'>{{ error.as_ref().unwrap() }}</p>{% endif %}<form class='stack-form' method='post' action='/register'><label>E-mail<input name='email' type='email' placeholder='voce@exemplo.com' autocomplete='email' required></label><label>Senha<input name='password' type='password' placeholder='Mínimo de 8 caracteres' autocomplete='new-password' minlength='8' required></label><button class='button button-primary' type='submit'>Criar conta <span aria-hidden='true'>→</span></button></form><p class='auth-switch'>Já possui uma conta? <a href='/login'>Entrar agora</a></p></div></section></main></body></html>", ext="html")]
struct RegisterPage { error: Option<String> }

#[derive(Template)]
#[template(source="<!doctype html><html lang='pt-BR'><head><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>Visão geral | Atlas</title><link rel='stylesheet' href='/static/app.css'></head><body class='app-page'><header class='topbar'><a class='brand' href='/'><span class='brand-mark'>A</span><span>atlas<span class='brand-dot'>.</span></span></a><div class='account'><span class='avatar'>{{ email.chars().next().unwrap_or('U') }}</span><span class='account-email'>{{ email }}</span><form method='post' action='/logout'><button class='logout-button' type='submit'>Sair</button></form></div></header><main class='dashboard-shell'><div class='page-heading'><div><p class='eyebrow'>Visão geral</p><h1>Sua carteira</h1><p class='muted'>Acompanhe sua posição e mantenha o controle das suas decisões.</p></div><span class='live-pill'><span class='status-dot'></span> Dados atualizados</span></div><section class='summary-grid' aria-label='Resumo da carteira'><article class='summary-card summary-card-featured'><div class='card-label'><span class='card-icon'>↗</span> Saldo investido</div><strong class='summary-value'>R$ {{ total }}</strong><span class='summary-note'>Compras menos vendas</span></article><article class='summary-card'><div class='card-label'><span class='card-icon card-icon-light'>◈</span> Ativos acompanhados</div><strong class='summary-value'>{{ assets }}</strong><span class='summary-note'>Posições na carteira</span></article><article class='summary-card summary-card-note'><span class='quote-mark'>“</span><p>Invista com contexto.<br><strong>Decida com calma.</strong></p></article></section><section class='content-grid'><div class='panel transactions-panel'><div class='panel-heading'><div><p class='eyebrow'>Histórico</p><h2>Movimentações recentes</h2></div><span class='count-badge'>{{ transactions.len() }}</span></div>{% if transactions.is_empty() %}<div class='empty-state'><span class='empty-icon'>+</span><h3>Comece a registrar</h3><p>Suas compras e vendas aparecerão aqui.</p></div>{% else %}<div class='table-wrap'><table><thead><tr><th>Ativo</th><th>Operação</th><th>Quantidade</th><th>Preço</th></tr></thead><tbody>{% for x in transactions %}<tr><td><span class='asset-symbol'>{{ x.symbol }}</span></td><td><span class='type-badge type-{{ x.kind|lower }}'>{{ x.kind }}</span></td><td>{{ x.quantity }}</td><td class='price'>R$ {{ x.price }}</td></tr>{% endfor %}</tbody></table></div>{% endif %}</div><aside class='panel form-panel'><div class='panel-heading'><div><p class='eyebrow'>Nova entrada</p><h2>Registrar operação</h2></div></div><form class='transaction-form' method='post' action='/transactions'><label>Ativo<input name='symbol' list='available-assets' placeholder='Ex.: PETR4' autocomplete='off' required></label><datalist id='available-assets'>{% for asset in assets_available %}<option value='{{ asset.symbol }}'>{{ asset.name }} · {{ asset.category }}</option>{% endfor %}</datalist><label>Operação<select name='kind'><option value='BUY'>Compra</option><option value='SELL'>Venda</option></select></label><label>Quantidade<input name='quantity' type='number' min='0.0001' step='0.0001' placeholder='0,0000' required></label><label>Preço unitário<input name='price' type='number' min='0.01' step='0.01' placeholder='R$ 0,00' required></label><button class='button button-primary' type='submit'>Salvar operação <span aria-hidden='true'>→</span></button></form></aside></section><section class='panel assets-panel'><div class='panel-heading'><div><p class='eyebrow'>Catálogo inicial</p><h2>Ativos disponíveis para registrar</h2><p class='muted panel-caption'>Lista de referência. Os preços devem ser informados conforme sua operação.</p></div><span class='reference-badge'>Referência</span></div><div class='available-assets'>{% for asset in assets_available %}<div class='asset-row'><span class='asset-symbol'>{{ asset.symbol }}</span><span class='asset-name'>{{ asset.name }}</span><span class='asset-category'>{{ asset.category }}</span></div>{% endfor %}</div></section></main></body></html>", ext="html")]
struct Dashboard { email: String, total: String, assets: i64, transactions: Vec<TxView>, assets_available: Vec<AssetOption> }
struct TxView { symbol: String, kind: String, quantity: String, price: String }

struct AssetOption { symbol: &'static str, name: &'static str, category: &'static str }

fn available_assets() -> Vec<AssetOption> {
    vec![
        AssetOption { symbol: "PETR4", name: "Petrobras PN", category: "Ação" },
        AssetOption { symbol: "VALE3", name: "Vale ON", category: "Ação" },
        AssetOption { symbol: "ITUB4", name: "Itaú Unibanco PN", category: "Ação" },
        AssetOption { symbol: "BBAS3", name: "Banco do Brasil ON", category: "Ação" },
        AssetOption { symbol: "WEGE3", name: "Weg ON", category: "Ação" },
        AssetOption { symbol: "B3SA3", name: "B3 ON", category: "Ação" },
        AssetOption { symbol: "MXRF11", name: "Maxi Renda", category: "FII" },
        AssetOption { symbol: "HGLG11", name: "CSHG Logística", category: "FII" },
        AssetOption { symbol: "KNRI11", name: "Kinea Renda Imobiliária", category: "FII" },
        AssetOption { symbol: "XPLG11", name: "XP Log", category: "FII" },
        AssetOption { symbol: "VISC11", name: "Vinci Shopping Centers", category: "FII" },
        AssetOption { symbol: "BOVA11", name: "iShares Ibovespa", category: "ETF" },
        AssetOption { symbol: "IVVB11", name: "iShares S&P 500", category: "ETF" },
        AssetOption { symbol: "TESOURO_SELIC", name: "Tesouro Selic", category: "Renda fixa" },
        AssetOption { symbol: "CDB", name: "Certificado de Depósito Bancário", category: "Renda fixa" },
        AssetOption { symbol: "LCI", name: "Letra de Crédito Imobiliário", category: "Renda fixa" },
        AssetOption { symbol: "LCA", name: "Letra de Crédito do Agronegócio", category: "Renda fixa" },
    ]
}

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
    let total: rust_decimal::Decimal = sqlx::query_scalar("SELECT COALESCE(SUM(CASE WHEN kind = 'BUY' THEN quantity * price ELSE -(quantity * price) END), 0) FROM transactions WHERE user_id = $1").bind(user_id).fetch_one(&state.db).await.unwrap_or_default();
    let assets: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT symbol) FROM transactions WHERE user_id = $1").bind(user_id).fetch_one(&state.db).await.unwrap_or(0);
    let rows = sqlx::query("SELECT symbol, kind, quantity, price FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50").bind(user_id).fetch_all(&state.db).await.unwrap_or_default();
    let transactions = rows.into_iter().map(|r| TxView { symbol: r.get("symbol"), kind: r.get("kind"), quantity: r.get::<rust_decimal::Decimal,_>("quantity").to_string(), price: r.get::<rust_decimal::Decimal,_>("price").to_string() }).collect();
    Html(Dashboard { email, total: total.to_string(), assets, transactions, assets_available: available_assets() }.render().unwrap()).into_response()
}

async fn create_transaction(headers: HeaderMap, State(state): State<AppState>, Form(form): Form<TransactionForm>) -> Response {
    let Some((user_id, _)) = current_user(&headers, &state.db).await else { return Redirect::to("/login").into_response(); };
    if form.quantity <= rust_decimal::Decimal::ZERO || form.price <= rust_decimal::Decimal::ZERO || !matches!(form.kind.as_str(), "BUY" | "SELL") { return StatusCode::BAD_REQUEST.into_response(); }
    let _ = sqlx::query("INSERT INTO transactions (id, user_id, symbol, kind, quantity, price) VALUES ($1,$2,$3,$4,$5,$6)").bind(Uuid::new_v4()).bind(user_id).bind(form.symbol.trim().to_uppercase()).bind(form.kind).bind(form.quantity).bind(form.price).execute(&state.db).await;
    Redirect::to("/").into_response()
}
