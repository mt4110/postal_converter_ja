# Postal Converter JA – Automatic Japanese Postal Code Updater

![Version](https://img.shields.io/badge/version-0.8.0-blue.svg)
![Status](https://img.shields.io/badge/status-beta-orange.svg)

Example Demo: [GitHub Pages](https://mt4110.github.io/postal_converter_ja/) / Support: [SPONSORS.md](../SPONSORS.md) / Plan: [V0_7_TO_V1_EXECUTION_PLAN.md](./V0_7_TO_V1_EXECUTION_PLAN.md)

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

GitHub Pages demo: **https://mt4110.github.io/postal_converter_ja/**

The Pages build uses `NEXT_PUBLIC_DEMO_MODE=true`, so the hosted sample can be explored with bundled demo data even when no public API is available. For live API integration, run the API locally or provide a public `NEXT_PUBLIC_API_URL`.

For the first publish, set `Settings > Pages > Source` to `GitHub Actions`, then run the `Frontend Pages` workflow manually.

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
👉 Kubernetes deployment guide (v0.8.2): [KUBERNETES_DEPLOYMENT.md](./KUBERNETES_DEPLOYMENT.md)
👉 Kubernetes adoption blueprint (existing platform ops): [K8S_ADOPTION_BLUEPRINT.md](./K8S_ADOPTION_BLUEPRINT.md)
👉 Kubernetes skeletons: `deploy/helm/postal-converter-ja` (Helm / default), `deploy/k8s/base` (Kustomize), `deploy/argocd` (ArgoCD route)

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
- [x] Deployment baseline (v0.8): AWS-first with GitHub Actions + Terraform, plus GCP/Azure target skeletons
- [x] Kubernetes minimum integration (Helm/Kustomize/ArgoCD)

### v0.8.x Focus (Deployment baseline)

- [x] AWS-first IaC flow: GitHub Actions + Terraform `validate/plan/apply` for dev
- [x] Environment split: `dev/stg/prod` AWS tfvars
- [x] Offline verification route: `plan` works without AWS secret in skeleton mode
- [x] Rollback runbook: `destroy` command path with evidence
- [x] Kubernetes minimum skeleton: Helm + Kustomize + ArgoCD scaffolding

### Human/Auth-Gated Remaining Tasks

- Enable GitHub Pages source as GitHub Actions in repository settings
- Enable the GitHub Sponsors profile for the `mt4110` account
- Configure real cloud accounts, OIDC roles, and GitHub Secrets/Variables before Terraform apply/destroy
- Complete v0.9.2 human QA evidence according to `docs/V0_9_0_ACCEPTANCE.md`

---

## ❤️ Sponsor & Partnerships

This project aims to improve access to accurate, modernized Japanese address data.

### 📮 To Japan Post Co., Ltd.

We believe this system can serve as an improved interface for the official postal datasets.  
If this aligns with your mission, we welcome sponsorship or a technical partnership.
