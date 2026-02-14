# ArgoCD Route (v0.8.2)

Apply the Application resource:

```bash
kubectl apply -f deploy/argocd/application-postal-converter-ja.yaml
```

This tracks:

- repo: `https://github.com/mt4110/postal_converter_ja.git`
- path: `deploy/helm/postal-converter-ja`
- targetRevision: `main`

For branch validation, update `targetRevision` to your feature branch.
