dev:
  mprocs -c mprocs.yaml

mprocs:
  ./scripts/mprocs-nix.sh dev-fed mprocs-new.yaml

test-ts:
  bun run wrappers/fedimint-ts/tests/test.ts

test-py:
  python3.11 wrappers/fedimint-py/test.py

test-py-async:
  python3.11 wrappers/fedimint-py/test_async.py

test-go:
  cd wrappers/fedimint-go && go run cmd/main.go

wscat:
  mprocs -c mprocs-wscat.yaml
