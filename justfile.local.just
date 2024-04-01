dev:
  mprocs -c mprocs.yaml

mprocs:
  ./scripts/mprocs-nix.sh dev-fed mprocs-new.yaml

test-ts:
  bun run wrappers/fedimint-ts/test.ts

test-go:
  cd wrappers/fedimint-go && go run cmd/main.go

test-go-pps:
  cd wrappers/fedimint-go && go test pkg/fedimint/client_test.go

wscat:
  mprocs -c mprocs-wscat.yaml
