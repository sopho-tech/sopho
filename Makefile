.PHONY: install run dev

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
	GOOGLE_REDIRECT_URI="http://localhost:3000" \
	ENVIRONMENT="development" \
	COOKIE_DOMAIN="localhost" \
	COOKIE_SECURE="false" \
	cargo run --release

# Dev: Run frontend and backend as separate processes
dev:
	@echo "Starting backend and frontend in development mode..."
	@trap 'kill 0' EXIT; \
	(cd backend && \
	GOOGLE_REDIRECT_URI="http://localhost:3000" \
	ENVIRONMENT="development" \
	COOKIE_DOMAIN="localhost" \
	COOKIE_SECURE="false" \
	cargo run) & \
	(cd frontend && \
	VITE_API_HOSTNAME=http://localhost:8000 \
	npm run dev) & \
	wait

