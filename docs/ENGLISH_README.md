# Postal Converter JA – Automatic Japanese Postal Code Updater

![Version](https://img.shields.io/badge/version-0.6.0-blue.svg)
![Status](https://img.shields.io/badge/status-beta-orange.svg)

**Postal Converter JA** is a fully automated system that fetches and updates the latest Japanese postal code data from Japan Post.  
It consists of a **Rust-based backend (Crawler + API)** and a **Next.js frontend**, providing a high-performance, up-to-date address lookup service.

---

## ✨ Features

- **Automatic Updates**  
  The crawler regularly downloads CSV files from Japan Post (default: every 24 hours) and updates the database.

- **Incremental / Differential Updates**  
  Automatically handles new postal codes, modifications, and discontinued entries.

- **High-Performance API**  
  Rust (Axum) backend delivers fast postal-code and address lookups.

- **Modern Frontend**  
  Next.js + TypeScript + Tailwind CSS demo application included.

- **Nix-Powered Environment**  
  Fully reproducible development setup using Nix.

- **Dual Database Support**  
  Works with both **MySQL** and **PostgreSQL**.  
  Selectable via environment variables.

---

## 🏛 Architecture

- **Frontend:** Next.js (React), TypeScript, Tailwind CSS, Radix UI
- **API Backend:** Rust (Axum), `tokio-postgres`, `mysql_async`
- **Crawler:** Rust, Tokio, Reqwest, CSV parser
- **Database:** MySQL / PostgreSQL (switchable)
- **Infrastructure:** Docker Compose (DB), Nix (toolchain)

---

## 📦 Prerequisites

You will need:

- **Nix** (for development environment)
- **Docker** (for running the databases)
- **Note**: Node.js / Yarn / Go / Rust are provided by `nix develop`

### Install Nix

```bash
# Official installer
sh <(curl -L https://nixos.org/nix/install)

# Or Determinate Systems installer (recommended)
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

---

## 🚀 Setup & Run

**Important:** Follow the steps in order.

### 1. Start Databases (Docker)

```bash
docker-compose up -d
docker ps
```

This launches:

| DB         | Port     |
| :--------- | :------- |
| MySQL      | **3204** |
| PostgreSQL | **3205** |

By default, Docker **named volumes** are used (recommended for stable local behavior).  
Use bind mounts only when needed for local inspection:

```bash
docker compose -f docker-compose.yml -f docker-compose.local.yml up -d
```

If local DB state is broken (e.g. `binlog.index Permission denied`), reset volumes first:

```bash
docker compose down -v
```

Optional Redis cache:

```bash
docker compose --profile cache up -d redis
```

### 2. Create Environment Files

```bash
# Crawler
cp worker/crawler/.env.example worker/crawler/.env

# API
cp worker/api/.env.example worker/api/.env
```

Set the database type in `.env`:

```bash
DATABASE_TYPE=postgres   # default
# DATABASE_TYPE=mysql
# DATABASE_TYPE=sqlite
SQLITE_DATABASE_PATH=storage/sqlite/postal_codes.sqlite3

# Optional Redis cache
REDIS_URL=redis://127.0.0.1:3206
REDIS_CACHE_TTL_SECONDS=300
```

> [!NOTE]
> `DATABASE_TYPE=sqlite` is for API read-only PoC. Direct SQLite writes from crawler are not supported.

### SQLite DB Build (PoC)

Generate SQLite DB from PostgreSQL data:

```bash
nix develop --command bash -lc "./scripts/build_sqlite_from_postgres.sh"
```

To package distributable SQLite artifacts (DB + checksum + manifest):

```bash
nix develop --command bash -lc "./scripts/package_sqlite_release.sh"
```

Generated files are placed in `artifacts/sqlite/`.

### 3. Run the Crawler (initial import + scheduled updates)

```bash
nix develop --command bash -lc "cd worker/crawler && cargo run --release --bin crawler"
```

Run only one cycle (for CI/batch jobs):

```bash
nix develop --command bash -lc "cd worker/crawler && CRAWLER_RUN_ONCE=true cargo run --release --bin crawler"
```

On first run:

- Downloads official Japan Post CSVs
- Inserts ~120,000 records
- Sets up daily auto-update task
- If `REDIS_URL` is set, Redis cache is invalidated after update

### 4. Run the API Server

```bash
nix develop --command bash -lc "cd worker/api && cargo run --release --bin api"
```

API starts at: **http://localhost:3202**

### 5. Run Frontend

```bash
nix develop --command bash -lc "cd frontend && yarn install && yarn dev"
```

Demo available at: **http://localhost:3203**

---

## 🛠 API Documentation

Full API endpoints and examples:  
👉 [API_SPEC.md](../API_SPEC.md) _(Note: Currently in Japanese only. Please use a translation tool.)_

OpenAPI JSON: `http://localhost:3202/openapi.json`
Swagger UI: `http://localhost:3202/docs`

## 🐛 Troubleshooting

See:  
👉 [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) _(Note: Currently in Japanese only.)_

## 🧑‍💻 Developer Notes

Docs for developers:  
👉 [DEVELOPMENT.md](./DEVELOPMENT.md) _(Note: Currently in Japanese only.)_
👉 [SQLITE_READONLY_POC.md](./SQLITE_READONLY_POC.md)
👉 `.github/workflows/sqlite-release.yml` (manual workflow for SQLite release artifacts)

---

## 🔐 License & Commercial Use

This project uses **Dual Licensing**:

### ✔ Free for:

- Individual use
- Non-profit
- Open source contributions

Licensed under **MIT License**.

### ✔ Commercial / Enterprise use:

If used inside a company, SaaS, or commercial service, a commercial license or GitHub Sponsors support is requested.

_(Current preview version: free for evaluation.)_

This model ensures sustainable development and fair support for long-term usage.

---

## 🗺 Roadmap

- [x] CI/CD (GitHub Actions)
- [x] Integration tests for MySQL/PostgreSQL
- [x] Lightweight Docker images (multi-stage for API/Crawler)
- [x] OpenAPI/Swagger documentation
- [x] Onboarding automation (`scripts/setup_nix_docker.sh`, `scripts/onboard.sh`)
- [ ] Multi-platform deployment baseline (GitHub Actions + Terraform)
- [ ] Kubernetes integration (Helm/Kustomize)

### v0.7.0 Focus (Sales-ready onboarding)

- [x] SDK samples for 3 use cases: EC checkout, member registration, call-center input support
- [x] Delivery checklist for contractor onboarding (`docs/CONTRACTOR_ONBOARDING_CHECKLIST.md`)
- [ ] Fresh-machine validation flow for install + start
- [x] Monthly SQLite release operation checklist (`docs/SQLITE_MONTHLY_OPERATION_CHECKLIST.md`)

---

## ❤️ Sponsor & Partnerships

This project aims to improve access to accurate, modernized Japanese address data.

### 📮 To Japan Post Co., Ltd.

We believe this system can serve as an improved interface for the official postal datasets.  
If this aligns with your mission, we welcome sponsorship or a technical partnership.
