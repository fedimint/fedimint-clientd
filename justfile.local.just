dev:
  mprocs -c mprocs.yaml

dev-regtest:
  ./scripts/mprocs-nix.sh dev-fed mprocs-new.yaml

test-ts:
  bun run wrappers/fedimint-ts/tests/test.ts

test-py:
  python3.11 wrappers/fedimint-py/test.py

test-py-async:
  python3.11 wrappers/fedimint-py/test_async.py

test-go:
  cd wrappers/fedimint-go && go run cmd/main.go

test-kotlin:
  cd wrappers/fedimint-kotlin && ./gradlew build && ./gradlew run

test-all:
  set -e
  (just test-ts && echo "test-ts completed successfully") &
  (just test-py && echo "test-py completed successfully") &
  (just test-py-async && echo "test-py-async completed successfully") &
  (just test-go && echo "test-go completed successfully") &
  (just test-kotlin && echo "test-kotlin completed successfully") &
  wait

wscat:
  mprocs -c mprocs-wscat.yaml

publish:
  cargo publish -p multimint && cargo publish -p fedimint-clientd && cargo publish -p clientd-stateless

test-invoice amountMsat:
  ./scripts/mutinynet-invoice.sh {{amountMsat}}
