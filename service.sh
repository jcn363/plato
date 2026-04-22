#! /bin/sh

if [ $# -lt 1 ]; then
	printf 'Usage: %s CMD [OPTS].\n' "${0##*/}" 1>&2
	exit 1
fi

CMD=$1
shift

	case "$CMD" in
	run_emulator)
		RUSTFLAGS="-L $PWD/libs_host" cargo run -p emulator "$@"
		;;
	install_importer)
		cargo install --path crates/importer "$@"
		;;
	*)
		printf 'Unknown command: %s.\n' "$CMD" 1>&2
		exit 1
		;;
esac
