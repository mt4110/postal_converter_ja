# Helm Skeleton (v0.8.2)

Minimal chart for API deployment.

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
