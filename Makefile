.DEFAULT_GOAL := tmux

run:
	./caby-service/scripts/run-dev.sh

tmux:
	tmux new-session -s caby-dev -d ./caby-service/scripts/run-dev.sh \; split-window -h ./caby-web/scripts/run-dev.sh \; attach