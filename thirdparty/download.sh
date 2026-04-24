#! /bin/bash

# Download all libraries by default if no arguments provided
if [ $# -eq 0 ]; then
	set --
fi

download_lib() {
	name=$1
	url=$2
	echo "Downloading ${name}..."
	# Save build-ios.sh if it exists
	if [ -f "$name/build-ios.sh" ]; then
		mkdir -p ../tmp
		mv "$name/build-ios.sh" "../tmp/${name}-build-ios.sh.bak"
	fi
	# Remove directory completely
	rm -rf "$name"
	mkdir "$name"
	curl -L -o "${name}.tgz" "$url"
	tar -xz --strip-components 1 -C "$name" -f "${name}.tgz" && rm "${name}.tgz"
	# Restore build-ios.sh if it was saved
	if [ -f "../tmp/${name}-build-ios.sh.bak" ]; then
		mv "../tmp/${name}-build-ios.sh.bak" "$name/build-ios.sh"
	fi
}

for name in "$@" ; do
	case "$name" in
		*)
			echo "Unknown library: ${name}." 1>&2
			exit 1
			;;
	esac
done
