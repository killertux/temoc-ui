#!/bin/sh
# Installs the latest temoc-ui release.
#
#   curl -fsSL https://raw.githubusercontent.com/killertux/temoc-ui/main/install.sh | sh
#
# Pass options after `sh -s --`:
#
#   curl -fsSL .../install.sh | sh -s -- --tui
#
#   --gui            Install the build with the window and the terminal (default)
#   --tui            Install the smaller build with only the terminal interface
#   --version TAG    Install a specific release, such as v0.1.0
#   --dir DIR        Install into DIR (default: ~/.local/bin)

set -eu

repo="killertux/temoc-ui"
tmp=""

fail() {
	echo "temoc-ui install: $1" >&2
	exit 1
}

usage() {
	cat <<EOF
Installs the latest temoc-ui release.

  --gui            Install the build with the window and the terminal (default)
  --tui            Install the smaller build with only the terminal interface
  --version TAG    Install a specific release, such as v0.1.0
  --dir DIR        Install into DIR (default: ~/.local/bin)
EOF
}

detect_target() {
	os="$(uname -s)"
	arch="$(uname -m)"
	case "$os" in
	Darwin)
		# A shell running under Rosetta reports x86_64 on Apple silicon.
		if [ "$arch" = arm64 ] || [ "$(sysctl -n hw.optional.arm64 2>/dev/null || echo 0)" = 1 ]; then
			echo aarch64-apple-darwin
		else
			fail "Intel Macs are not supported. Use cargo install instead."
		fi
		;;
	Linux)
		case "$arch" in
		x86_64 | amd64) echo x86_64-unknown-linux-gnu ;;
		aarch64 | arm64) echo aarch64-unknown-linux-gnu ;;
		*) fail "unsupported Linux architecture: $arch" ;;
		esac
		;;
	*) fail "unsupported operating system: $os" ;;
	esac
}

verify_checksum() {
	# The checksum file names the asset, so this has to run next to it.
	if command -v sha256sum >/dev/null 2>&1; then
		(cd "$tmp" && sha256sum -c "$1.sha256" >/dev/null) || fail "checksum mismatch for $1"
	elif command -v shasum >/dev/null 2>&1; then
		(cd "$tmp" && shasum -a 256 -c "$1.sha256" >/dev/null) || fail "checksum mismatch for $1"
	else
		fail "neither sha256sum nor shasum is available to verify the download"
	fi
}

main() {
	variant="gui"
	version="latest"
	dir="$HOME/.local/bin"

	while [ $# -gt 0 ]; do
		case "$1" in
		--gui) variant="gui" ;;
		--tui) variant="tui" ;;
		--version)
			[ $# -ge 2 ] || fail "--version needs a value"
			version="$2"
			shift
			;;
		--dir)
			[ $# -ge 2 ] || fail "--dir needs a value"
			dir="$2"
			shift
			;;
		-h | --help)
			usage
			return 0
			;;
		*) fail "unknown option: $1" ;;
		esac
		shift
	done

	command -v curl >/dev/null 2>&1 || fail "curl is required"
	command -v tar >/dev/null 2>&1 || fail "tar is required"

	target="$(detect_target)"
	asset="temoc-ui-$target-$variant.tar.gz"
	if [ "$version" = latest ]; then
		base="https://github.com/$repo/releases/latest/download"
	else
		base="https://github.com/$repo/releases/download/$version"
	fi

	tmp="$(mktemp -d)"
	trap 'rm -rf "$tmp"' EXIT

	echo "Downloading $asset ($version)"
	curl -fsSL -o "$tmp/$asset" "$base/$asset" || fail "could not download $base/$asset"
	curl -fsSL -o "$tmp/$asset.sha256" "$base/$asset.sha256" || fail "could not download the checksum for $asset"
	verify_checksum "$asset"

	tar -xzf "$tmp/$asset" -C "$tmp" temoc-ui
	mkdir -p "$dir"
	# Moving a new file into place works even while an old temoc-ui is running.
	cp "$tmp/temoc-ui" "$dir/temoc-ui.new"
	chmod 755 "$dir/temoc-ui.new"
	mv -f "$dir/temoc-ui.new" "$dir/temoc-ui"

	echo "Installed temoc-ui to $dir/temoc-ui"
	case ":$PATH:" in
	*":$dir:"*) ;;
	*) echo "Add $dir to your PATH to run it." ;;
	esac
	if ! command -v temoc >/dev/null 2>&1; then
		echo "Running tests also needs temoc: https://github.com/killertux/temoc/releases"
	fi
}

main "$@"
