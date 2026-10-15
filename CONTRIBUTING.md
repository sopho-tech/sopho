# Contributing to Sopho

## Prerequisites

- [rustup](https://rustup.rs). The toolchain version comes from `backend/rust-toolchain.toml`.
- The cargo tools: `make tools` installs cargo-nextest, cargo-llvm-cov, cargo-deny and cargo-shear.
- Node 26, as pinned in `.nvmrc`.
- Docker, for the backend tests (they query a Postgres sample database even when the metadata database is SQLite), the end-to-end suite and the image tests.
- [prek](https://github.com/j178/prek), which runs the git hooks.

## Setup

```bash
make setup
make hooks
```

`make setup` installs the npm dependencies of the root tooling, `frontend`, `docs` and `e2e`, fetches the cargo dependencies and installs Chromium for Playwright. `make hooks` installs the git hooks.

| Hook stage | Runs                                                                                                                                                                                                  |
| ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| pre-commit | The formatters and linters on the staged files: rustfmt, Prettier, tombi, Oxlint, stylelint, markdownlint, typos, actionlint, zizmor, hadolint, shellcheck, shfmt, gitleaks and the whitespace fixers |
| commit-msg | commitlint                                                                                                                                                                                            |
| pre-push   | For the areas the pushed commits change: clippy and the backend unit tests, the frontend `tsc -b` and Vitest `unit` project, the docs type check and the e2e type check                               |

Fixers rewrite files and fail the commit, so review their changes, stage them and commit again. `git commit --no-verify` skips the hooks for the rare commit that must.

`make lint`, `make typecheck` and `make fmt` run the same checks and formatters over the whole repository.

## Tests

| Layer               | Command                      | Runs                                                                                                                |
| ------------------- | ---------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| Backend             | `make test-backend`          | Unit tests and the API integration tests with a SQLite metadata database                                            |
| Backend on Postgres | `make test-backend-postgres` | The same tests with a Postgres metadata database                                                                    |
| Frontend            | `make test-frontend`         | The Vitest `unit` (Node) and `browser` (Chromium) projects                                                          |
| End-to-end          | `make test-e2e`              | Builds `frontend`, `sopho` and `fake-llm`, then runs the Playwright `native` project                                |
| Image               | `make test-docker`           | Builds `sopho:dev`, runs the `@smoke` specs against it on SQLite and Postgres, and upgrades from the latest release |
| Everything          | `make check`                 | `lint`, `typecheck` and `test`                                                                                      |

- `TEST_DB` selects the metadata database of the backend tests, `sqlite` or `postgres`. `backend/.cargo/config.toml` sets `sqlite` for local runs.
- The Postgres-backed tests share one `postgres:17` container, which Testcontainers starts on first use. `TEST_POSTGRES_URL` points them at an existing server instead, for example `postgres://postgres:postgres@localhost:5432/postgres`.
- `make test-clean` removes the shared test container.
- The end-to-end suite serves Sopho on port 18080 and the fake LLM on port 18081. Free both ports before `make test-e2e`.
- `make test-docker` pulls `ghcr.io/sopho-tech/sopho/sopho:latest` before the upgrade specs, so they always start from the latest release.
- `make test-image SOPHO_IMAGE=<image> SOPHO_UPGRADE_FROM_IMAGE=<image>` tests an image you built yourself. Set `SOPHO_UPGRADE_FROM_IMAGE=none` only in a release that resets databases.

### Where new tests go

- Backend unit tests live in `#[cfg(test)]` modules next to the code. The text-to-SQL prompt builders have insta snapshots; after an intended prompt change, rerun the tests with `INSTA_UPDATE=always` and review the snapshot diff.
- Backend API tests live in `backend/tests/api/`, one module per area. `support/` starts the app (`TestApp::spawn`), signs in and creates connections.
- Frontend tests sit next to the code: `*.test.ts` runs in Node and `*.test.tsx` runs in Chromium. Mock the API with MSW handlers keyed by `API_ENDPOINTS` (`src/test/msw/handlers.ts`), and render with `renderWithProviders`. A request without a handler fails the test.
- End-to-end specs live in `e2e/specs/`. Tag the flows that must also work in the Docker image with `{ tag: "@smoke" }`. Upgrade specs live in `e2e/upgrade_specs/`.

### Fake LLM fixtures

The backend tests and the end-to-end suite answer LLM requests with the fake LLM in `backend/fake-llm`. Each fixture is `backend/fake-llm/fixtures/<provider>/<agent>.json`:

- `match`: text that appears only in that agent's requests, such as the JSON schema title of a typed prompt (`"title":"RouterDecision"`) or a phrase from its prompt.
- `exclude` (optional): text that rules the fixture out, for agents whose later rounds repeat the earlier ones.
- `response`: the response body in the provider's format.

Exactly one fixture must match each request. A request that no fixture matches, or that several match, fails the test. `fail_next_request` makes the next request for a fixture fail with a server error or malformed JSON.

## Coverage

`make coverage` writes `backend/lcov.info` and `frontend/coverage/lcov.info`. CI uploads both to Codecov. Coverage is informational: it never fails a build, and tests are not written to raise the number.

## Commits

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org), and commitlint checks them.

## Editor setup

- The Oxlint and Prettier extensions.
- The TypeScript 7 language service from the workspace `typescript` package.
- rust-analyzer with `rust-analyzer.check.command` set to `clippy`.

## Troubleshooting

- prek refuses to install hooks while `core.hooksPath` is set. Run `git config --unset-all --local core.hooksPath`, then `make hooks`.
- Testcontainers cannot reach Docker with OrbStack or colima. Export `DOCKER_HOST` (OrbStack: `unix://$HOME/.orbstack/run/docker.sock`; colima: `unix://$HOME/.colima/default/docker.sock`) and `TESTCONTAINERS_DOCKER_SOCKET_OVERRIDE=/var/run/docker.sock`.
