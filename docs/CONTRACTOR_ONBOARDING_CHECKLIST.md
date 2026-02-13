# Contractor Onboarding Checklist (v0.7.0)

Last updated: 2026-02-13 (JST)

## 1) Preconditions

- Git clone completed for `postal_converter_ja`
- Nix installed and shell profile loaded
- Docker engine available (`docker ps` succeeds)
- Ports available: `3202` (API), `3203` (Frontend), `3205` (PostgreSQL), `3206` (Redis)

## 2) First-Time Setup

1. Run automated setup:
   - `./scripts/setup_nix_docker.sh --profile dev --skip-onboard`
2. Start local demo stack:
   - `./scripts/onboard.sh --profile demo`
3. Confirm service URLs:
   - API: `http://127.0.0.1:3202/health`
   - Swagger: `http://127.0.0.1:3202/docs`
   - Frontend: `http://127.0.0.1:3203`

## 3) Verification Points (Demo Day)

- API basic lookup:
  - `curl -fsS http://127.0.0.1:3202/postal_codes/1000001`
- API partial address search:
  - `curl -fsS "http://127.0.0.1:3202/postal_codes/search?address=新宿&mode=partial&limit=5"`
- Swagger UI opens without errors
- Frontend shows 3 sample tabs:
  - EC checkout form
  - Member registration form
  - Call-center input support form
- Redis cache invalidation path is configured when `REDIS_URL` is set

## 4) Customer Handoff Artifacts

- Environment command list used in onboarding
- `.env` parameter sheet (non-secret defaults only)
- API integration examples (`lookupZip`, `searchAddress`)
- Current data version evidence (`data_update_audits` record)
- Rollback command template (`worker/crawler/src/bin/rollback.rs`)

## 5) Exit Criteria

- Customer can run local demo with documented commands only
- At least one address lookup flow succeeds in front of customer
- Handoff artifacts are attached to delivery note
