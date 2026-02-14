# Helm Skeleton (v0.8.3)

Minimal chart for API deployment.

Default behavior in v0.8.3:

- plain-text Secret creation is disabled (`secret.create=false`)
- use External Secrets by enabling `externalSecret.enabled=true`

## Lint

```bash
helm lint deploy/helm/postal-converter-ja
```

## Install

```bash
helm upgrade --install postal-converter-ja deploy/helm/postal-converter-ja \
  --namespace postal-converter-ja \
  --create-namespace
```

## External Secrets Example

Review and customize:

```bash
cat deploy/helm/postal-converter-ja/values.external-secrets.example.yaml
```

Render with ExternalSecret enabled:

```bash
helm template postal-converter-ja deploy/helm/postal-converter-ja \
  --namespace postal-converter-ja \
  -f deploy/helm/postal-converter-ja/values.external-secrets.example.yaml
```
