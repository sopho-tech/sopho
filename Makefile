.PHONY: install run dev tools setup hooks fmt lint lint-repo lint-backend typecheck

CARGO_TOOLS := cargo-nextest cargo-llvm-cov cargo-deny cargo-shear

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
	cd backend && cargo fetch

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
