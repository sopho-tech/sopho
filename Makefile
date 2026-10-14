.PHONY: install run dev tools setup hooks fmt lint lint-repo lint-backend typecheck test test-backend test-backend-postgres test-frontend test-e2e test-clean coverage coverage-backend coverage-frontend check

CARGO_TOOLS := cargo-nextest cargo-llvm-cov cargo-deny cargo-shear
BACKEND_COVERAGE_IGNORE := (src/entity/|migration/|fake-llm/)

# Installation: Build both frontend and backend
install:
	@echo "Installing frontend dependencies..."
	cd frontend && npm install
	@echo "Building frontend..."
	cd frontend && npm run build
	@echo "Building backend..."
	cd backend && cargo build --release
	@echo "Installation complete!"

# Run: Start backend with compiled frontend (production mode)
run:
	@echo "Starting backend server..."
	cd backend && \
	ENVIRONMENT="development" \
	COOKIE_DOMAIN="localhost" \
	COOKIE_SECURE="false" \
	FRONTEND_DIR="../frontend/dist" \
	cargo run --release

# Dev: Run frontend and backend as separate processes
dev:
	@echo "Starting backend and frontend in development mode..."
	@trap 'kill 0' EXIT; \
	(cd backend && \
	ENVIRONMENT="development" \
	COOKIE_DOMAIN="localhost" \
	COOKIE_SECURE="false" \
	FRONTEND_DIR="../frontend/dist" \
	cargo run) & \
	(cd frontend && \
	VITE_API_HOSTNAME=http://localhost:8000 \
	npm run dev) & \
	wait

tools:
	cd backend && cargo install --locked $(CARGO_TOOLS)

setup:
	@for cargo_tool in $(CARGO_TOOLS); do \
		command -v $$cargo_tool >/dev/null || { echo "$$cargo_tool is missing. Run: make tools"; exit 1; }; \
	done
	npm ci
	cd frontend && npm ci
	cd docs && npm ci
	cd e2e && npm ci
	cd backend && cargo fetch
	cd frontend && npx --no-install playwright install chromium
	cd e2e && npx --no-install playwright install chromium

hooks:
	prek install

fmt:
	cd backend && cargo fmt --all
	npx --no-install prettier --write .
	npx --no-install tombi format

lint: lint-repo lint-backend

lint-repo:
	prek run --all-files --show-diff-on-failure

lint-backend:
	cd backend && cargo clippy --workspace --all-targets --locked -- -D warnings
	cd backend && cargo deny check bans licenses sources
	cd backend && cargo shear

typecheck:
	cd frontend && npx tsc -b
	cd docs && npm run types:check
	cd e2e && npx --no-install tsc --noEmit

test: test-backend test-frontend

test-backend:
	cd backend && TEST_DB=sqlite cargo nextest run --workspace

test-backend-postgres:
	cd backend && TEST_DB=postgres cargo nextest run --workspace

test-frontend:
	cd frontend && npx --no-install vitest run

test-e2e:
	cd frontend && npx --no-install vite build
	cd backend && cargo build -p sopho -p fake-llm
	cd e2e && npx --no-install playwright test --project=native

test-clean:
	docker rm --force --volumes sopho-test-postgres

coverage: coverage-backend coverage-frontend

coverage-backend:
	cd backend && cargo llvm-cov nextest --workspace --lcov --output-path lcov.info --ignore-filename-regex '$(BACKEND_COVERAGE_IGNORE)'

coverage-frontend:
	cd frontend && npx --no-install vitest run --coverage --reporter=default --reporter=junit --outputFile.junit=junit.xml

check: lint typecheck test
