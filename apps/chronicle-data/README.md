# chronicle-data

Data platform: ingestion, lake storage, metadata governance.

### Stack
- Apache Beam (Go/Python)
- Iceberg + BigLake
- RLAC + column-level encryption

### How to Build
```sh
bazel build //apps/chronicle-data:chronicle_pipeline
