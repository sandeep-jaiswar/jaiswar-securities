# helix-portal

Client-facing real-time portal with federated GraphQL APIs.

### Stack
- React + Astro + WASM
- Edge functions (Cloudflare Workers)
- CDN + geo-aware routing

### How to Build
```sh
bazel build //apps/helix-portal:helix_frontend
