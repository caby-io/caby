.DEFAULT_GOAL := tmux

.PHONY: bootstrap init-service init-service-tools init-web init-config run-service run-web tmux debug-mobile-win debug-mobile-win-cleanup debug-mobile-adb debug-mobile-adb-cleanup

WSL_IP ?= $(shell hostname -I 2>/dev/null | awk '{print $$1}')
ADB ?= adb.exe
MOBILE_PORTS := 5173 8080 1411

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

debug-mobile-win:
	@echo ">>> debug-mobile-win — exposing dev ports to LAN via Windows portproxy + firewall"
	@echo "WSL IP: $(WSL_IP)"
	gsudo.exe "$$(wslpath -w scripts/debug-mobile-win.bat)" $(WSL_IP)

debug-mobile-win-cleanup:
	@echo ">>> debug-mobile-win-cleanup — removing portproxy + firewall rules"
	gsudo.exe "$$(wslpath -w scripts/debug-mobile-win-cleanup.bat)"

debug-mobile-adb:
	@echo ">>> debug-mobile-adb — adb reverse for dev ports"
	@for port in $(MOBILE_PORTS); do \
		$(ADB) reverse tcp:$$port tcp:$$port; \
	done

debug-mobile-adb-cleanup:
	@echo ">>> debug-mobile-adb-cleanup — removing adb reverse mappings"
	@for port in $(MOBILE_PORTS); do \
		$(ADB) reverse --remove tcp:$$port || true; \
	done
