# Atlas Investment Wallet

Carteira de investimentos fullstack construída em Rust. O Atlas permite criar uma conta, registrar compras e vendas e acompanhar uma visão consolidada das posições em uma interface web renderizada no servidor.

## O que está incluído

- Cadastro, login e logout
- Senhas protegidas com Argon2
- Sessões persistidas no PostgreSQL com cookie HttpOnly
- Isolamento das movimentações por usuário
- Registro de operações de compra e venda
- Dashboard com total investido, ativos acompanhados e histórico
- Migrations executadas automaticamente pelo SQLx
- Valores financeiros representados com `Decimal`

## Pré-requisitos

- [Rust e Cargo](https://www.rust-lang.org/tools/install)
- [Docker Desktop](https://www.docker.com/products/docker-desktop/)
- Git, caso o projeto seja clonado

## Instalação no Windows

O projeto extraído deste ZIP pode conter uma pasta interna com o mesmo nome. Use sempre a pasta que contém diretamente `Cargo.toml` e `docker-compose.yml`:

```powershell
cd "C:\Users\Pauli\Downloads\investment-wallet-main\investment-wallet-main"
```

Crie o arquivo de ambiente a partir do exemplo:

```powershell
Copy-Item .env.example .env
```

O `.env` deve conter:

```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/investment_wallet
RUST_LOG=investment_wallet=debug,tower_http=debug
```

### Corrigir o erro `docker-credential-desktop`

Se o Docker exibir `error getting credentials` ou `docker-credential-desktop executable file not found`, remova o credential store quebrado da configuração local. O comando abaixo cria um backup antes da alteração:

```powershell
$configPath = Join-Path $HOME ".docker\config.json"
Copy-Item $configPath "$configPath.bak" -Force
$config = Get-Content $configPath -Raw | ConvertFrom-Json
$config.PSObject.Properties.Remove("credsStore")
$config | ConvertTo-Json -Depth 10 | Set-Content $configPath -Encoding utf8
```

Se o Docker reclamar de BOM/encoding depois disso, regrave o arquivo assim:

```powershell
$json = Get-Content $configPath -Raw | ConvertFrom-Json | ConvertTo-Json -Depth 10
[IO.File]::WriteAllText($configPath, $json, [Text.UTF8Encoding]::new($false))
```

## Executar

Inicie o PostgreSQL:

```powershell
docker compose up -d postgres
```

Confira o serviço:

```powershell
docker compose ps
```

Inicie a aplicação:

```powershell
cargo run
```

Abra http://localhost:3000. Na primeira visita, crie uma conta e depois registre suas operações.

As migrations em `migrations/` são aplicadas automaticamente quando o servidor inicia.

## Comandos úteis

```powershell
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run
```

Para parar somente o banco:

```powershell
docker compose stop postgres
```

Para parar e remover o container, preservando os dados do volume:

```powershell
docker compose down
```

Para apagar também os dados persistidos do PostgreSQL:

```powershell
docker compose down -v
```

## Arquitetura

- **Axum**: rotas HTTP e handlers
- **Tokio**: runtime assíncrono
- **SQLx + PostgreSQL**: persistência e migrations
- **Askama**: renderização server-side dos templates
- **Argon2**: hash seguro de senhas
- **Tower HTTP**: arquivos estáticos e tracing

## Estrutura principal

```text
src/main.rs              Rotas, autenticação e regras da aplicação
templates/               Templates HTML Askama
static/app.css           Estilos da interface
migrations/              Schema inicial e autenticação
docker-compose.yml       PostgreSQL para desenvolvimento
.env.example              Variáveis de ambiente de referência
```

## Licença

Este projeto está licenciado sob a [MIT License](LICENSE).
