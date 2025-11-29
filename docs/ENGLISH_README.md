# Postal Converter JA – Automatic Japanese Postal Code Updater

![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)
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
- **Mise** (optional, recommended for Node.js/Yarn version management)

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
```

### 3. Run the Crawler (initial import + scheduled updates)

```bash
cd worker/crawler
nix develop
cargo run --release --bin crawler
```

On first run:

- Downloads official Japan Post CSVs
- Inserts ~120,000 records
- Sets up daily auto-update task

### 4. Run the API Server

```bash
cd worker/api
nix develop
cargo run --release --bin api
```

API starts at: **http://localhost:3202**

### 5. Run Frontend

```bash
cd frontend
yarn install
yarn dev
```

Demo available at: **http://localhost:3203**

---

## 🛠 API Documentation

Full API endpoints and examples:  
👉 [API_SPEC.md](../API_SPEC.md) _(Note: Currently in Japanese only. Please use a translation tool.)_

## 🐛 Troubleshooting

See:  
👉 [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) _(Note: Currently in Japanese only.)_

## 🧑‍💻 Developer Notes

Docs for developers:  
👉 [DEVELOPMENT.md](./DEVELOPMENT.md) _(Note: Currently in Japanese only.)_

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

- [ ] CI/CD (GitHub Actions)
- [ ] Integration tests for MySQL/PostgreSQL
- [ ] Lightweight Docker images
- [ ] OpenAPI/Swagger documentation

---

## ❤️ Sponsor & Partnerships

This project aims to improve access to accurate, modernized Japanese address data.

### 📮 To Japan Post Co., Ltd.

We believe this system can serve as an improved interface for the official postal datasets.  
If this aligns with your mission, we welcome sponsorship or a technical partnership.
