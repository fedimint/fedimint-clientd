dev:
  mprocs -c mprocs.yaml

mprocs:
  ./scripts/mprocs-nix.sh dev-fed mprocs-new.yaml

test-ts:
  bun run wrappers/fedimint-ts/test.ts

test-go:
  cd wrappers/fedimint-go && go run cmd/main.go
