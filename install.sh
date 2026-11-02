#!/usr/bin/env sh
set -eu

APP_NAME="matsurikafetch"
BIN_ALIAS="mfetch"
PREFIX="${HOME}/.local"
CONFIG_DIR="${XDG_CONFIG_HOME:-${HOME}/.config}/mfetch"
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

if [ "$(uname -s)" != "Linux" ]; then
    echo "Matsurikafetch currently supports Linux only." >&2
    exit 1
fi

if ! command -v cargo >/dev/null 2>&1 || ! command -v rustc >/dev/null 2>&1; then
    cat >&2 <<'EOF'
Rust and Cargo are required to build Matsurikafetch.
Install the stable Rust toolchain from https://rustup.rs/ and run this script again.
EOF
    exit 1
fi

RUST_VERSION=$(rustc --version | awk '{print $2}')
echo "Building ${APP_NAME} with Rust ${RUST_VERSION}..."
cargo install --path "$SCRIPT_DIR" --root "$PREFIX" --force

mkdir -p "$PREFIX/bin" "$CONFIG_DIR"
if [ ! -f "$CONFIG_DIR/config.jsonc" ]; then
    cp "$SCRIPT_DIR/config.jsonc" "$CONFIG_DIR/config.jsonc"
    echo "Created $CONFIG_DIR/config.jsonc"
else
    echo "Keeping existing config: $CONFIG_DIR/config.jsonc"
fi

if [ -x "$PREFIX/bin/$APP_NAME" ]; then
    ln -sf "$APP_NAME" "$PREFIX/bin/$BIN_ALIAS"
fi

case ":${PATH}:" in
    *":$PREFIX/bin:"*) ;;
    *)
        echo
        echo "Add this to your shell profile if ~/.local/bin is not already in PATH:"
        echo '  export PATH="$HOME/.local/bin:$PATH"'
        ;;
esac

echo
echo "Installed successfully."
echo "Run: $APP_NAME   (or: $BIN_ALIAS)"
echo "Config: $CONFIG_DIR/config.jsonc"
