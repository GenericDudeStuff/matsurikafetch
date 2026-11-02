# Matsurikafetch

A Linux system-information fetch utility written in Rust with the help of AI, inspired by Fastfetch
and Matsurika Shinouji from *Maria†Holic*.

The default output follows the layout of the supplied Fastfetch configuration:
a title and separator, OS/kernel/package/shell/WM/terminal information, a
second group for media/battery/uptime/age, hardware information, and a colour
swatch row. The supplied transparent Matsurika artwork is the default logo.

## Install (any Linux distribution)

Requirements:

- Linux
- Rust stable and Cargo (Rust 1.80 or newer)
- A terminal with Unicode support; a Nerd Font is recommended for the icons

Extract the project, then run:

```sh
cd matsurikafetch
./install.sh
```

The installer builds the project using Cargo, installs the executable to
`~/.local/bin/matsurikafetch`, creates the `mfetch` alias, and creates
`~/.config/mfetch/config.jsonc` if it does not already exist. It never
overwrites an existing configuration.

If `~/.local/bin` is not in your `PATH`, add this to your shell profile:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

Run either command:

```sh
mfetch
# or
matsurikafetch
```

To update, download/extract the newer source and run `./install.sh` again.

## Uninstall

From the project directory, run:

```sh
./uninstall.sh
```

This removes the installed executable and the `mfetch` alias, but keeps your
configuration. To remove the configuration directory as well, explicitly run:

```sh
./uninstall.sh --purge
```

## Build manually

```sh
cargo build --release
./target/release/matsurikafetch
```

## Configuration

Configuration is JSONC (JSON with comments and trailing commas):

```text
~/.config/mfetch/config.jsonc
```

The installer places a starter config there. Main settings:

```jsonc
{
  "logo": {
    "source": "matsurika", // or "auto"
    "height": 24,
    "padding": { "top": 2, "right": 3 }
  },
  "display": {
    "separator": "  ➜  ",
    "color": { "keys": "white", "title": "white" },
    "key": { "width": 14 }
  },
  "color": true,
  "nerd_font": true,
  "image_protocol": "auto"
}
```

- `logo.source = "matsurika"` uses the bundled transparent image.
- `logo.source = "auto"` uses a built-in distribution logo.
- `logo.height` sets the image height in terminal rows.
- `logo.padding.top` and `logo.padding.right` control spacing.
- `display.separator` sets the key/value separator.
- `display.key.width` controls the label column width.
- `modules` controls module order. Supported types: `title`, `separator`,
  `break`, `os`, `kernel`, `packages`, `shell`, `wm`, `terminal`, `media`,
  `battery`, `uptime`, `age`, `cpu`, `gpu`, `storage`, `memory`, `colors`.
- `nerd_font = false` removes Nerd Font symbols from default labels.
- `image_protocol` accepts `auto`, `kitty`, `iterm`, or `halfblocks`.
  `auto` delegates graphics capability detection to `viuer`; if no native
  protocol is available, it uses half-block rendering.

The config parser accepts JSONC comments and trailing commas. Unknown fields
are ignored so Fastfetch-style metadata such as `$schema` and logo colour
settings can coexist with Matsurikafetch settings.

## Modules and platform notes

Matsurikafetch currently targets Linux. It reports:

- OS, host model, kernel, uptime and package count
- Shell, desktop/window-manager environment and terminal
- CPU model, thread count and sampled utilization
- GPU model (NVIDIA via `nvidia-smi`; otherwise DRM/sysfs and optional `lspci`)
- Root filesystem storage, battery when present, and RAM
- A best-effort root filesystem age value

Media-player integration is currently a placeholder (`Nothing playing`).
Battery is omitted if no battery is exposed by `/sys/class/power_supply`.
Package counts currently support Gentoo, pacman and dpkg databases; RPM
currently reports database presence rather than an exact package count.

## Development with Vim

```sh
vim src/main.rs
vim config.jsonc
cargo run
```

The image is bundled at `assets/matsurika.png`, so the executable does not
depend on the source tree or a separate image file after installation.
