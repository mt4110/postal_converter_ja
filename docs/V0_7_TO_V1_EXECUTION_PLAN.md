# v0.7.0 to v1.0.0 Execution Plan

Last updated: 2026-02-13 (JST)

## Scope

- v0.6.0 merged on 2026-02-13
- This document fixes priorities from v0.7.0 to v1.0.0 for sales-ready delivery

## Version Strategy

### v0.7.0 (Onboarding Acceleration)

Goal:
- Make first customer onboarding reproducible in one day.

Deliverables:
- 3 SDK sample flows (EC, Member Registration, Call Center)
- One-command setup with profile-based start/stop
- SQLite release packaging checklist for monthly operations
- Customer onboarding checklist (technical + operational)

Exit Criteria:
- A fresh machine can run local demo with documented commands only.
- SDK sample flows are runnable and cover core search behaviors.

### v0.8.0 (Deployment Baseline)

Goal:
- Establish repeatable cloud deployment path.

Deliverables:
- GitHub Actions + Terraform baseline fixed on one primary cloud target
- OIDC auth and environment separation (dev/stg/prod)
- IaC validation and plan/apply guardrails

Exit Criteria:
- `plan` and `apply` are reproducible from CI for dev.
- Deployment rollback path is documented and tested once.

### v0.9.0 (Operations Readiness)

Goal:
- Standardize production operations and incident response.

Deliverables:
- SLO/SLI dashboard and alert thresholds
- Runbook for update failure, cache incident, DB rollback
- Audit report template for monthly data update operations

Exit Criteria:
- Simulated incident can be handled with runbook only.
- Monthly operation report is generated from defined inputs.

### v1.0.0 (GA)

Goal:
- Release as commercially deployable baseline.

Deliverables:
- Stable release policy and compatibility policy
- Security hardening review results
- Final release notes and migration guidance

Exit Criteria:
- GA checklist is complete and signed off.
- Version `v1.0.0` can be tagged with reproducible release steps.

## Current Sprint (v0.7.0)

- [x] Add call-center SDK sample to frontend showcase
- [x] Add contractor onboarding checklist doc
- [x] Add monthly SQLite release operation checklist doc
- [ ] Validate fresh machine onboarding flow and record evidence

## Next One Task

1. Execute a fresh-machine rehearsal and attach command output evidence to PR notes.
2. Confirm host-level `curl` checks from the same terminal session used to run `onboard.sh`.
