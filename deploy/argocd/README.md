# ArgoCD Route (v0.8.3)

Apply the Application resource:

```bash
kubectl apply -f deploy/argocd/application-postal-converter-ja.yaml
```

This creates 3 ArgoCD Applications:

- `postal-converter-ja-dev` -> namespace `postal-converter-ja-dev` -> `values-dev.yaml`
- `postal-converter-ja-stg` -> namespace `postal-converter-ja-stg` -> `values-stg.yaml`
- `postal-converter-ja-prod` -> namespace `postal-converter-ja-prod` -> `values-prod.yaml`

Shared source configuration:

- repo: `https://github.com/mt4110/postal_converter_ja.git`
- path: `deploy/helm/postal-converter-ja`
- targetRevision: `main`

Environment defaults:

- dev: `replicaCount=1`
- stg: `replicaCount=1`
- prod: `replicaCount=2`

For branch validation, update `targetRevision` to your feature branch.
