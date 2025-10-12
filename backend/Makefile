.PHONY: dev-db dev-full dev-stop

run:
	cargo run

dev-db:
	docker-compose -f docker-compose.yaml up

dev-full:
	docker-compose -f docker-compose.yaml -f docker-compose.override.yaml up

dev-stop:
	docker-compose -f docker-compose.yaml -f docker-compose.override.yaml down
