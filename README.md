# investment-wallet
O que o projeto faz

O Investment Wallet é uma aplicação Fullstack desenvolvida em Rust para gerenciamento de uma carteira de investimentos.

A aplicação permite que usuários:

Criem uma conta e façam login;
Tenham suas sessões protegidas;
Registrem operações de compra e venda;
Consultem suas movimentações;
Acompanhem seus ativos e posições;
Armazenem os dados de forma persistente no PostgreSQL;
Acessem uma interface web renderizada no servidor com Askama.

A arquitetura foi pensada para separar autenticação, regras de negócio, persistência e apresentação.

Como executar a aplicação

Pré-requisitos:

Rust e Cargo
Docker e Docker Compose
PostgreSQL, caso não utilize o ambiente Docker

Clone o projeto:

git clone https://github.com/PauloGuerra2019/investment-wallet.git
cd investment-wallet

Configure as variáveis de ambiente:

DATABASE_URL=postgres://postgres:postgres@localhost:5432/investment_wallet
RUST_LOG=info

Suba o PostgreSQL:

docker compose up -d postgres

Execute a aplicação:

cargo run

A aplicação ficará disponível em:

http://localhost:3000

As migrations do banco são executadas automaticamente durante a inicialização.

Tecnologias utilizadas
Tecnologia	Utilização
Rust	Linguagem principal
Axum	Framework HTTP/API
Tokio	Runtime assíncrono
PostgreSQL	Banco de dados
SQLx	Acesso ao banco e migrations
Askama	Renderização de páginas HTML
Argon2	Hash seguro das senhas
Serde	Serialização e desserialização
Tower HTTP	Middleware HTTP e arquivos estáticos
Tracing	Logs estruturados
Docker	Ambiente de execução
GitHub	Versionamento e colaboração
Qual melhoria foi implementada

A principal melhoria implementada foi a autenticação completa de usuários.

A aplicação inicialmente permitia trabalhar com movimentações sem estabelecer uma identidade para o usuário. A melhoria introduziu:

Tabela de usuários;
Cadastro;
Login;
Logout;
Hash de senha utilizando Argon2;
Sessões persistidas no PostgreSQL;
Cookies HttpOnly;
Expiração das sessões;
Middleware de autenticação;
Associação das movimentações ao usuário autenticado;
Proteção das informações da carteira.

Isso permite que cada usuário tenha acesso somente aos próprios dados.

Além disso, a aplicação utiliza tipos decimais para valores financeiros, evitando trabalhar com f64 para cálculos monetários.

Como testar

Para verificar se o projeto está funcionando:

cargo check

Executar os testes:

cargo test

Verificar problemas apontados pelo Clippy:

cargo clippy --all-targets --all-features -- -D warnings

Executar a aplicação:

cargo run

Depois acessar:

http://localhost:3000

O fluxo básico de teste é:

Cadastro
   ↓
Login
   ↓
Dashboard
   ↓
Adicionar compra
   ↓
Consultar movimentação
   ↓
Logout
   ↓
Tentar acessar área protegida
   ↓
Redirecionamento para login
O que aprendi durante o desafio

Durante o desenvolvimento do projeto, os principais aprendizados foram relacionados à construção de uma aplicação completa utilizando Rust.

Rust no desenvolvimento web

Aprendi a estruturar uma aplicação web utilizando Axum e Tokio, trabalhando com programação assíncrona e o sistema de tipos do Rust.

Persistência de dados

O projeto permitiu trabalhar com PostgreSQL através do SQLx, incluindo criação de tabelas, relacionamentos, índices e migrations.

Autenticação e segurança

Um dos principais aprendizados foi entender que autenticação não envolve apenas criar uma tela de login. É necessário trabalhar corretamente com:

Hash de senhas;
Sessões;
Cookies;
Expiração;
Middleware;
Autorização;
Isolamento dos dados entre usuários.

Server-side rendering

O uso do Askama mostrou uma abordagem diferente das aplicações SPA tradicionais, permitindo gerar HTML no próprio backend utilizando templates fortemente integrados ao Rust.

Modelagem financeira

Também foi necessário considerar um ponto importante em aplicações financeiras: precisão numérica. Valores monetários não devem depender de operações com ponto flutuante sem controle adequado.

Arquitetura Fullstack

Por fim, o desafio mostrou como integrar diferentes camadas:

                    ┌──────────────┐
                    │    Browser   │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │     Axum     │
                    └──────┬───────┘
                           │
             ┌─────────────┴─────────────┐
             ▼                           ▼
      ┌─────────────┐             ┌─────────────┐
      │   Askama    │             │     API     │
      └─────────────┘             └──────┬──────┘
                                         │
                                         ▼
                                  ┌─────────────┐
                                  │    SQLx     │
                                  └──────┬──────┘
                                         │
                                         ▼
                                  ┌─────────────┐
                                  │ PostgreSQL  │
                                  └─────────────┘

Esse projeto consolidou conhecimentos de Back-End, banco de dados, autenticação, segurança, HTML server-side, APIs e arquitetura de aplicações em Rust.
## Licença

Este projeto está licenciado sob a [MIT License](LICENSE).
