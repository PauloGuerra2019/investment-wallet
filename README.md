# Atlas Investment Wallet

Uma carteira de investimentos fullstack construída em Rust para registrar operações e acompanhar uma visão consolidada do patrimônio. A aplicação usa renderização server-side, PostgreSQL e autenticação por sessão.

## O que o projeto faz

O Atlas permite que cada usuário:

- Crie uma conta, faça login e encerre a sessão;
- Tenha a senha protegida com hash Argon2;
- Registre operações de compra e venda;
- Consulte o histórico das próprias movimentações;
- Acompanhe o saldo líquido investido e a quantidade de ativos;
- Escolha ativos a partir de um catálogo de referência com ações, FIIs, ETFs e renda fixa;
- Armazene os dados de forma persistente no PostgreSQL.

O saldo líquido considera o valor das compras menos o valor das vendas. Por exemplo, uma compra de R$ 1.000 e uma venda de R$ 400 resultam em R$ 600 de saldo investido.

> O catálogo de ativos é uma lista de referência para facilitar o registro. A aplicação não consulta cotações em tempo real; o preço da operação deve ser informado pelo usuário.

## Como executar a aplicação

### Pré-requisitos

- [Rust e Cargo](https://www.rust-lang.org/tools/install);
- [Docker Desktop](https://www.docker.com/products/docker-desktop/);
- Git, caso o projeto seja clonado.

### 1. Clonar o projeto

```powershell
git clone https://github.com/PauloGuerra2019/investment-wallet.git
cd investment-wallet
```

Se o projeto tiver sido extraído de um ZIP com uma pasta duplicada, entre na pasta que contém diretamente `Cargo.toml` e `docker-compose.yml`:

```powershell
cd "C:\Users\Pauli\Downloads\investment-wallet-main\investment-wallet-main"
```

### 2. Configurar as variáveis de ambiente

Crie o arquivo `.env` a partir do exemplo:

```powershell
Copy-Item .env.example .env
```

O arquivo deve conter:

```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/investment_wallet
RUST_LOG=investment_wallet=debug,tower_http=debug
```

O `.env` é local e não deve ser enviado ao GitHub.

### 3. Iniciar o PostgreSQL

```powershell
docker compose up -d postgres
docker compose ps
```

O serviço ficará disponível em `localhost:5432`. As migrations são executadas automaticamente pela aplicação ao iniciar.

### 4. Iniciar o servidor

```powershell
cargo run
```

Acesse http://localhost:3000, crie uma conta e registre suas operações.

### Solução para `docker-credential-desktop`

Se o Docker exibir `error getting credentials` ou informar que `docker-credential-desktop` não foi encontrado, faça backup da configuração e remova somente o credential store:

```powershell
$configPath = Join-Path $HOME ".docker\config.json"
Copy-Item $configPath "$configPath.bak" -Force
$config = Get-Content $configPath -Raw | ConvertFrom-Json
$config.PSObject.Properties.Remove("credsStore")
$json = $config | ConvertTo-Json -Depth 10
[IO.File]::WriteAllText($configPath, $json, [Text.UTF8Encoding]::new($false))
```

Depois repita `docker compose up -d postgres`.

## Tecnologias utilizadas

| Tecnologia | Utilização |
| --- | --- |
| Rust | Linguagem principal e regras da aplicação |
| Axum | Framework HTTP e roteamento |
| Tokio | Runtime assíncrono |
| PostgreSQL | Banco de dados relacional |
| SQLx | Queries, conexão e migrations |
| Askama | Renderização server-side dos templates HTML |
| Argon2 | Hash seguro das senhas |
| Rust Decimal | Precisão para valores financeiros |
| Serde | Desserialização dos formulários |
| Tower HTTP | Arquivos estáticos e tracing de requisições |
| Docker Compose | Ambiente local do PostgreSQL |

## Qual melhoria foi implementada

A principal melhoria foi transformar o fluxo de movimentações em uma carteira com autenticação e experiência de uso completa.

### Autenticação e segurança

- Cadastro de usuários;
- Login e logout;
- Hash de senha com Argon2;
- Sessões persistidas no PostgreSQL;
- Cookies `HttpOnly` com expiração;
- Proteção do dashboard para usuários autenticados;
- Isolamento das movimentações por usuário.

### Controle financeiro

- Registro de compras e vendas;
- Cálculo líquido de compras menos vendas;
- Uso de `Decimal` em vez de `f64` para valores financeiros;
- Histórico das últimas 50 movimentações;
- Catálogo de referência para ações, FIIs, ETFs e renda fixa.

### Interface

- Dashboard responsivo;
- Resumo de saldo e quantidade de ativos;
- Badges visuais para compra e venda;
- Formulários com validação de campos;
- Estado vazio para carteiras sem movimentações;
- Catálogo com preenchimento assistido do ticker;
- Layout de login e cadastro com identidade visual própria.

## Como testar sua versão

### Verificação de compilação

```powershell
cargo check
```

### Testes automatizados

```powershell
cargo test
```

### Análise estática com Clippy

```powershell
cargo clippy --all-targets --all-features -- -D warnings
```

### Teste manual do fluxo principal

1. Inicie o PostgreSQL com Docker Compose;
2. Execute `cargo run`;
3. Acesse http://localhost:3000/register;
4. Crie uma conta com senha de pelo menos oito caracteres;
5. Registre uma compra, por exemplo, `PETR4`, quantidade `10` e preço `30`;
6. Registre uma venda do mesmo ativo, por exemplo, quantidade `2` e preço `30`;
7. Confira no dashboard se o saldo considera compras menos vendas;
8. Verifique se as movimentações aparecem no histórico;
9. Faça logout e confirme o redirecionamento para `/login`;
10. Tente acessar `/` sem sessão e confirme que a área protegida não fica disponível.

### Health check

Com a aplicação em execução, teste:

```powershell
curl http://localhost:3000/health
```

Resposta esperada:

```text
ok
```

## O que foi aprendido durante o desafio

### Rust no desenvolvimento web

O projeto ajudou a consolidar a estrutura de uma aplicação web com Axum e Tokio, incluindo rotas, handlers assíncronos, extração de formulários e gerenciamento explícito de estado.

### Persistência de dados

Foi possível praticar a integração do PostgreSQL com SQLx, a organização de migrations, relacionamentos entre usuários e movimentações e a criação de índices para consultas frequentes.

### Autenticação não é apenas uma tela de login

Um fluxo de autenticação completo exige hash de senhas, sessões, cookies, expiração, logout, proteção das rotas e isolamento dos dados de cada usuário.

### Renderização server-side

O Askama mostrou como gerar HTML no backend com templates integrados ao código Rust, mantendo o fluxo simples e sem a necessidade de uma SPA para esta aplicação.

### Precisão em aplicações financeiras

Valores monetários não devem depender de operações com ponto flutuante sem controle. O uso de `rust_decimal` evita perdas de precisão no cálculo de compras e vendas.

### Organização de uma aplicação fullstack

O desafio conectou apresentação, autenticação, regras de negócio, persistência e infraestrutura local em um único fluxo executável.

## Estrutura principal

```text
src/main.rs              Rotas, autenticação e regras da aplicação
templates/               Templates HTML Askama
static/app.css           Estilos da interface
migrations/              Schema inicial e autenticação
docker-compose.yml       PostgreSQL para desenvolvimento
.env.example             Variáveis de ambiente de referência
```

## Comandos úteis

Parar apenas o PostgreSQL:

```powershell
docker compose stop postgres
```

Parar os serviços e preservar o volume de dados:

```powershell
docker compose down
```

Para apagar também os dados persistidos, use:

```powershell
docker compose down -v
```

## Licença

Este projeto está licenciado sob a [MIT License](LICENSE).
