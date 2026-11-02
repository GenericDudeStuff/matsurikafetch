#!/usr/bin/env sh
set -eu

APP_NAME="matsurikafetch"
BIN_ALIAS="mfetch"
PREFIX="${HOME}/.local"
BIN_DIR="${PREFIX}/bin"
CONFIG_DIR="${XDG_CONFIG_HOME:-${HOME}/.config}/mfetch"
PURGE_CONFIG=0

if [ "${1:-}" = "--purge" ]; then
    PURGE_CONFIG=1
elif [ "$#" -gt 0 ]; then
    echo "Usage: $0 [--purge]" >&2
    exit 2
fi

if [ "$(uname -s)" != "Linux" ]; then
    echo "Matsurikafetch currently supports Linux only." >&2
    exit 1
fi

# Remove the executable installed by install.sh.
if [ -e "$BIN_DIR/$APP_NAME" ] || [ -L "$BIN_DIR/$APP_NAME" ]; then
    rm -f "$BIN_DIR/$APP_NAME"
    echo "Removed $BIN_DIR/$APP_NAME"
else
    echo "Executable not found: $BIN_DIR/$APP_NAME"
fi

# Only remove the short alias if it points to Matsurikafetch.
if [ -L "$BIN_DIR/$BIN_ALIAS" ]; then
    alias_target=$(readlink "$BIN_DIR/$BIN_ALIAS" || true)
    case "$alias_target" in
        "$APP_NAME"|"$BIN_DIR/$APP_NAME")
            rm -f "$BIN_DIR/$BIN_ALIAS"
            echo "Removed $BIN_DIR/$BIN_ALIAS"
            ;;
        *)
            echo "Keeping $BIN_DIR/$BIN_ALIAS (it points to '$alias_target')"
            ;;
    esac
elif [ -e "$BIN_DIR/$BIN_ALIAS" ]; then
    echo "Keeping $BIN_DIR/$BIN_ALIAS (not a Matsurikafetch symlink)"
fi

if [ "$PURGE_CONFIG" -eq 1 ]; then
    if [ -d "$CONFIG_DIR" ]; then
        rm -rf "$CONFIG_DIR"
        echo "Removed configuration directory: $CONFIG_DIR"
    else
        echo "No configuration directory found: $CONFIG_DIR"
    fi
else
    echo "Keeping configuration and personal settings: $CONFIG_DIR"
    echo "To remove them too, run: $0 --purge"
fi

echo "Matsurikafetch uninstalled."
