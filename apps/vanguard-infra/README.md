# vanguard-infra

SRE, mesh networking, and observability infra stack.

### Stack
- GKE Autopilot
- Istio + Flagger + OTel
- Canary + SLO-based deploys

### Build
```sh
bazel build //apps/vanguard-infra:vanguard_mesh_config
