# Onboarding Rehearsal Evidence

- Executed at (JST): 2026-02-13 13:51:44 JST
- Branch: feature/v0.7.0
- Commit: 3176cc4

## 1) Start Command

```txt
./scripts/onboard.sh --profile dev
```

```txt
[onboard] Profile=dev: starting PostgreSQL + Redis + API + Frontend
 Network postal_converter_ja_default Creating 
 Network postal_converter_ja_default Created 
 Container postgres_container Creating 
 Container redis_container Creating 
 Container redis_container Created 
 Container postgres_container Created 
 Container redis_container Starting 
 Container postgres_container Starting 
 Container redis_container Started 
 Container postgres_container Started 
[onboard] PostgreSQL is ready.
[onboard] Starting API...
[onboard] API health is ready: http://127.0.0.1:3202/health
[onboard] Swagger UI is ready: http://127.0.0.1:3202/docs
[onboard] Starting Frontend...
[onboard] Frontend is ready: http://127.0.0.1:3203

Onboarding completed (profile: dev)

API:
  http://127.0.0.1:3202
Swagger:
  http://127.0.0.1:3202/docs
Frontend:
  http://127.0.0.1:3203

Logs:
  /tmp/postal_converter_ja_onboard/api.log
  /tmp/postal_converter_ja_onboard/frontend.log

Stop command:
  ./scripts/onboard.sh --stop
```

## 2) Connectivity Checks (same host terminal)

```txt
curl -sS -o /tmp/pcj_health.out -w '%{http_code}' http://127.0.0.1:3202/health
=> 200

curl -sS -o /tmp/pcj_openapi.out -w '%{http_code}' http://127.0.0.1:3202/openapi.json
=> 200

curl -sS -o /tmp/pcj_swagger.out -w '%{http_code}' http://127.0.0.1:3202/docs
=> 200

curl -sS -o /tmp/pcj_frontend.out -w '%{http_code}' http://127.0.0.1:3203
=> 200
```

## 3) Stop Command

```txt
./scripts/onboard.sh --stop
```

```txt
[onboard] Stopping managed local processes...
[onboard] Stopped. Docker services are down.
```
