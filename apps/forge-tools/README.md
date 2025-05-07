# forge-tools

Internal developer platform with GitOps, CI/CD, and tooling.

### Stack
- Bazel rules
- ArgoCD + GitHub Actions
- Policy-as-code

### Build
```sh
bazel build //apps/forge-tools:forge_ci_tools
