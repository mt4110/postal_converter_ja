# Kubernetes Base Skeleton (v0.8.2)

This directory provides a minimal Kustomize base for the API service.
For default deployment, use Helm chart at `deploy/helm/postal-converter-ja`.

## Apply

```bash
kubectl apply -k deploy/k8s/base
```

## Verify

```bash
kubectl -n postal-converter-ja get pods,svc
kubectl -n postal-converter-ja port-forward svc/postal-converter-api 3202:3202
curl -fsS http://127.0.0.1:3202/health
```

## Notes

- This is a starter skeleton, not production-hardening.
- Replace `image` in `deployment.yaml` before use.
- Replace values in `secret.yaml` for your environment.
