#! /bin/bash

# Download all libraries by default if no arguments provided
if [ $# -eq 0 ]; then
	set -- zlib bzip2 libpng libjpeg openjpeg jbig2dec freetype2 harfbuzz gumbo djvulibre mupdf
fi

download_lib() {
	name=$1
	url=$2
	echo "Downloading ${name}..."
	if [ -d "$name" ]; then
		git ls-files -o --directory -z "$name" | xargs -0 rm -rf
	else
		mkdir "$name"
	fi
	wget -q --show-progress -O "${name}.tgz" "$url"
	tar -xz --strip-components 1 -C "$name" -f "${name}.tgz" && rm "${name}.tgz"
}

for name in "$@" ; do
	case "$name" in
		zlib)
			download_lib zlib "https://www.zlib.net/zlib-1.3.1.tar.gz"
			;;
		bzip2)
			download_lib bzip2 "https://sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz"
			;;
		libpng)
			download_lib libpng "https://download.sourceforge.net/libpng/libpng-1.6.53.tar.gz"
			;;
		libjpeg)
			download_lib libjpeg "http://www.ijg.org/files/jpegsrc.v9f.tar.gz"
			;;
		openjpeg)
			download_lib openjpeg "https://github.com/uclouvain/openjpeg/archive/v2.5.4.tar.gz"
			;;
		jbig2dec)
			download_lib jbig2dec "https://github.com/ArtifexSoftware/jbig2dec/releases/download/0.20/jbig2dec-0.20.tar.gz"
			;;
		freetype2)
			download_lib freetype2 "https://download.savannah.gnu.org/releases/freetype/freetype-2.14.1.tar.gz"
			;;
		harfbuzz)
			download_lib harfbuzz "https://github.com/harfbuzz/harfbuzz/archive/12.3.0.tar.gz"
			;;
		gumbo)
			download_lib gumbo "https://github.com/google/gumbo-parser/archive/v0.10.1.tar.gz"
			;;
		djvulibre)
			download_lib djvulibre "http://downloads.sourceforge.net/djvu/djvulibre-3.5.29.tar.gz"
			;;
		mupdf)
			download_lib mupdf "https://casper.mupdf.com/downloads/archive/mupdf-1.27.0-source.tar.gz"
			;;
		*)
			echo "Unknown library: ${name}." 1>&2
			exit 1
			;;
	esac
done
