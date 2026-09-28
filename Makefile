# zkpool-solana — Makefile
#
# All commands assume the `solana` container is running.
#
# See `make help` for available targets.

SHELL := /bin/bash
COMPOSE := docker compose -f infra/docker-compose.yml
CONTAINER := solana

# Docker exec helpers.
# `-ic` is required — `.bashrc` is not read by `bash -c`.
exec_c := $(COMPOSE) exec $(CONTAINER) bash -ic
exec_d := $(COMPOSE) exec -d $(CONTAINER) bash -ic

.DEFAULT_GOAL := help

# ============================================================
# Help
# ============================================================

.PHONY: help
help:
	@echo "zkpool-solana — available targets"
	@echo ""
	@echo "  Lifecycle:"
	@echo "    make up          — start docker compose + merkle + prover + backend"
	@echo "    make down        — stop all services"
	@echo "    make reset       — stop, clear DB + Redis, restart"
	@echo "    make status      — show running services and their health"
	@echo "    make logs        — tail all service logs"
	@echo ""
	@echo "  Web:"
	@echo "    make web         — start Vite dev server (port 5173)"
	@echo ""
	@echo "  Build:"
	@echo "    make build       — build backend, prover, merkle, web"
	@echo "    make clean       — remove target/, node_modules/, dist/"
	@echo ""

# ============================================================
# Lifecycle
# ============================================================

.PHONY: up
up: docker-up merkle prover backend wait-healthy
	@echo ""
	@echo "✅ All services up."
	@echo "   backend  http://localhost:4001/api/health"
	@echo "   prover   http://localhost:4002/health"
	@echo "   merkle   http://localhost:4003/health"

.PHONY: docker-up
docker-up:
	@echo "→ docker compose up"
	@$(COMPOSE) up -d

.PHONY: merkle
merkle:
	@echo "→ starting merkle (4003)"
	@$(COMPOSE) exec -d $(CONTAINER) bash -ic 'pkill -f "node src/server.js" || true; cd /home/ubuntu/services/merkle && nohup node src/server.js > /tmp/merkle.log 2>&1 &'

.PHONY: prover
prover:
	@echo "→ starting prover (4002)"
	@$(COMPOSE) exec -d $(CONTAINER) bash -ic 'pkill -f zkpool-prover || true; cd /home/ubuntu/services/prover && nohup ./target/release/zkpool-prover > /tmp/prover.log 2>&1 &'

.PHONY: backend
backend:
	@echo "→ starting backend (4001)"
	@$(COMPOSE) exec -d $(CONTAINER) bash -ic 'pkill -f zkpool-backend || true; cd /home/ubuntu/services/backend && nohup ./target/release/zkpool-backend > /tmp/backend.log 2>&1 &'

.PHONY: wait-healthy
wait-healthy:
	@echo "→ waiting for services..."
	@sleep 3
	@$(exec_c) 'curl -sf -m 5 http://127.0.0.1:4003/health >/dev/null && echo "  ✓ merkle" || (echo "  ✗ merkle" && exit 1)'
	@$(exec_c) 'curl -sf -m 5 http://127.0.0.1:4002/health >/dev/null && echo "  ✓ prover" || (echo "  ✗ prover" && exit 1)'
	@$(exec_c) 'curl -sf -m 5 http://127.0.0.1:4001/api/health >/dev/null && echo "  ✓ backend" || (echo "  ✗ backend" && exit 1)'

.PHONY: down
down:
	@echo "→ stopping services"
	@$(exec_c) 'pkill -f "node src/server.js" || true' 2>/dev/null || true
	@$(exec_c) 'pkill -f zkpool-prover || true' 2>/dev/null || true
	@$(exec_c) 'pkill -f zkpool-backend || true' 2>/dev/null || true
	@$(COMPOSE) down 2>/dev/null || true
	@echo "done."

.PHONY: reset
reset: down
	@echo "→ docker compose up (infra only)"
	@$(COMPOSE) up -d
	@sleep 3
	@echo "→ clearing Postgres"
	@docker exec zkpool-postgres psql -U zkpool -d zkpool -c "DELETE FROM nullifiers; DELETE FROM roots; DELETE FROM commitments;" || true
	@echo "→ clearing Redis (tree:*, cache:*)"
	@docker exec zkpool-redis sh -c 'redis-cli --scan --pattern "tree:*" | xargs -r redis-cli DEL' || true
	@docker exec zkpool-redis sh -c 'redis-cli --scan --pattern "cache:*" | xargs -r redis-cli DEL' || true
	@docker exec zkpool-redis redis-cli DEL indexer:last_signature || true
	@$(MAKE) merkle prover backend wait-healthy
	@echo ""
	@echo "✅ All services up (fresh state)."

.PHONY: status
status:
	@echo "=== Containers ==="
	@$(COMPOSE) ps
	@echo ""
	@echo "=== Processes inside solana container ==="
	@$(exec_c) 'ps aux | grep -E "node src|zkpool-backend|zkpool-prover" | grep -v grep || echo "(none)"'
	@echo ""
	@echo "=== HTTP health ==="
	@$(exec_c) 'for p in 4001 4002 4003; do printf "  %s: " $$p; curl -sf -m 3 http://127.0.0.1:$$p/health || curl -sf -m 3 http://127.0.0.1:$$p/api/health || echo "DOWN"; echo; done'

.PHONY: logs
logs:
	@echo "=== merkle ==="
	@$(exec_c) 'tail -20 /tmp/merkle.log 2>/dev/null || echo "(no log)"'
	@echo ""
	@echo "=== prover ==="
	@$(exec_c) 'tail -20 /tmp/prover.log 2>/dev/null || echo "(no log)"'
	@echo ""
	@echo "=== backend ==="
	@$(exec_c) 'tail -20 /tmp/backend.log 2>/dev/null || echo "(no log)"'

# ============================================================
# Web
# ============================================================

.PHONY: web
web:
	@echo "→ starting Vite dev server (5173)"
	@$(COMPOSE) exec -d $(CONTAINER) bash -ic 'pkill -f vite || true; cd /home/ubuntu/web && nohup pnpm dev > /tmp/web.log 2>&1 &'
	@sleep 3
	@curl -sf -m 5 http://localhost:5173/ >/dev/null && echo "   ✓ http://localhost:5173" || echo "   ✗ vite not reachable"
	
# ============================================================
# Build
# ============================================================

.PHONY: build
build:
	@echo "→ building backend"
	@$(exec_c) 'cd /home/ubuntu/services/backend && cargo build --release'
	@echo "→ building prover"
	@$(exec_c) 'cd /home/ubuntu/services/prover && cargo build --release'
	@echo "→ building merkle (no build step, Node.js)"
	@echo "→ building web"
	@$(exec_c) 'cd /home/ubuntu/web && pnpm build'

.PHONY: clean
clean:
	@echo "→ removing build artifacts"
	@rm -rf services/backend/target services/prover/target services/merkle/node_modules web/node_modules web/dist
	@rm -rf scripts/*/target tests/target onchain/target
	@echo "done."
