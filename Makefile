.PHONY: help build up down logs test clean

help:
	@echo "Aureon Blockchain Docker Operations"
	@echo "===================================="
	@echo "make build              - Build Docker images"
	@echo "make build-dev          - Build development Docker image"
	@echo "make up                 - Start 3-node PoW cluster"
	@echo "make up-dev             - Start PoS development cluster"
	@echo "make down               - Stop all containers"
	@echo "make logs               - View container logs"
	@echo "make logs-node-1        - View Node 1 logs"
	@echo "make test               - Run integration tests"
	@echo "make test-dev           - Run dev tests"
	@echo "make clean              - Remove containers and volumes"
	@echo "make clean-images       - Remove Docker images"
	@echo "make status             - Show container status"
	@echo "make shell-node-1       - Open shell in Node 1"
	@echo "make health-check       - Check node health"

# Build operations
build:
	docker-compose build --no-cache

build-dev:
	docker-compose -f docker-compose.dev.yml build --no-cache

# Cluster operations
up:
	@echo "Starting 3-node Proof of Work cluster..."
	docker-compose up -d
	@echo "Waiting for nodes to start..."
	sleep 5
	@echo "Cluster started!"
	@echo "Node 1 API: http://localhost:8000"
	@echo "Node 2 API: http://localhost:8001"
	@echo "Node 3 API: http://localhost:8002"

up-dev:
	@echo "Starting PoS development cluster..."
	docker-compose -f docker-compose.dev.yml up -d
	@echo "Waiting for validators to start..."
	sleep 5
	@echo "Development cluster started!"
	@echo "Validator 1 API: http://localhost:8010"
	@echo "Validator 2 API: http://localhost:8011"
	@echo "Node API: http://localhost:8020"

down:
	docker-compose down
	docker-compose -f docker-compose.dev.yml down

logs:
	docker-compose logs -f

logs-node-1:
	docker-compose logs -f aureon-node-1

logs-node-2:
	docker-compose logs -f aureon-node-2

logs-node-3:
	docker-compose logs -f aureon-node-3

# Testing operations
test: up
	@echo "Running integration tests..."
	docker-compose exec aureon-node-1 curl -s http://localhost:8080/chain/head | jq .
	docker-compose exec aureon-node-2 curl -s http://localhost:8080/chain/head | jq .
	docker-compose exec aureon-node-3 curl -s http://localhost:8080/chain/head | jq .
	@echo "Tests completed"

test-dev: up-dev
	@echo "Running dev integration tests..."
	docker-compose -f docker-compose.dev.yml exec aureon-validator-1 curl -s http://localhost:8080/chain/head | jq .
	docker-compose -f docker-compose.dev.yml exec aureon-validator-2 curl -s http://localhost:8080/chain/head | jq .
	docker-compose -f docker-compose.dev.yml exec aureon-node curl -s http://localhost:8080/chain/head | jq .
	@echo "Dev tests completed"

# Cleanup operations
clean:
	docker-compose down -v
	docker-compose -f docker-compose.dev.yml down -v
	@echo "Containers and volumes cleaned"

clean-images:
	docker rmi aureon-chain:latest || true
	docker system prune -f
	@echo "Images cleaned"

# Monitoring operations
status:
	docker-compose ps
	docker-compose -f docker-compose.dev.yml ps

shell-node-1:
	docker-compose exec aureon-node-1 /bin/bash

shell-node-2:
	docker-compose exec aureon-node-2 /bin/bash

shell-node-3:
	docker-compose exec aureon-node-3 /bin/bash

health-check:
	@echo "Checking node health..."
	@docker-compose exec aureon-node-1 curl -s http://localhost:8080/chain/head && echo "✅ Node 1 healthy" || echo "❌ Node 1 unhealthy"
	@docker-compose exec aureon-node-2 curl -s http://localhost:8080/chain/head && echo "✅ Node 2 healthy" || echo "❌ Node 2 unhealthy"
	@docker-compose exec aureon-node-3 curl -s http://localhost:8080/chain/head && echo "✅ Node 3 healthy" || echo "❌ Node 3 unhealthy"

# Development operations
dev-build:
	cargo build --release -p aureon-node

dev-test:
	cargo test --all

dev-run:
	cargo run -p aureon-node
