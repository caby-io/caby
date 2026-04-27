.DEFAULT_GOAL := tmux

.PHONY: bootstrap init-service init-service-tools init-web init-config run-service run-web tmux

bootstrap: init-service init-service-tools init-web init-config

init-service:
	@echo ">>> init-service"
	@if [ ! -f caby-service/.env ]; then \
		cp caby-service/.env.example caby-service/.env; \
		echo "created caby-service/.env from .env.example"; \
	else \
		echo "caby-service/.env already exists, skipping"; \
	fi

init-service-tools:
	@echo ">>> init-service-tools"
	cargo install cargo-watch
	rustup toolchain install nightly --allow-downgrade -c rustfmt

init-web:
	@echo ">>> init-web"
	cd caby-web && pnpm install

init-config:
	@echo ">>> init-config"
	@CONFIG_PATH="$${CABY_HOME_PATH:-$$HOME/cabynet}/config.yaml"; \
	if [ -f "$$CONFIG_PATH" ]; then \
		echo "$$CONFIG_PATH already exists, skipping"; \
	else \
		mkdir -p "$$(dirname "$$CONFIG_PATH")"; \
		printf '%s\n' \
			'spaces:' \
			'  - name: home' \
			'    display: Home' \
			'' \
			'users: []' \
			> "$$CONFIG_PATH"; \
		echo "created $$CONFIG_PATH"; \
	fi

run-service:
	./caby-service/scripts/run-dev.sh

run-web:
	./caby-web/scripts/run-dev.sh

tmux:
	tmux new-session -s caby-dev -d ./caby-service/scripts/run-dev.sh \; split-window -h ./caby-web/scripts/run-dev.sh \; attach
