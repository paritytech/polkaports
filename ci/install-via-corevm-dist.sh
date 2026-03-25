#!/bin/sh

main() {
	set -ex
	workdir="$(mktemp -d)"
	trap cleanup EXIT
	start_http_server
	install_corevm_dist
}

start_http_server() {
	mkdir "$workdir"/srv
	python3 -m http.server -d "$workdir"/srv &
	touch "$workdir"/srv/health
	server_pid="$!"
	until curl --fail --no-progress-meter "http://127.0.0.1:8000/health" >/dev/null; do
		sleep 1
		printf "Waiting for HTTP server to start..." >&2
	done
	export COREVM_DIST_RELEASE_URL='http://127.0.0.1:8000/'
	b2sum *.tar.zst | tee crates/corevm-dist/b2sum.txt
    cp *.tar.zst "$workdir"/srv
}

install_corevm_dist() {
	cargo install --path crates/corevm-dist --root "$workdir"
	"$workdir"/bin/corevm-dist install
}

cleanup() {
	kill "$server_pid" 2>/dev/null || true
	wait
	rm -rf "$workdir"
}

main
