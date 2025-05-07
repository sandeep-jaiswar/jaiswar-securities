# hermes-settle

Post-trade and settlement workflow orchestration using Temporal.

### Stack
- Go + Java (optional)
- Temporal for workflows
- Idempotent event processing

### How to Build
```sh
bazel build //apps/hermes-settle:hermes_settle_service
