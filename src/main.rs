use serde::Deserialize;
use std::{
    env,
    fs,
    io::{self, IsTerminal, Write},
    path::Path,
    process::Command,
    thread,
    time::Duration,
};
use sysinfo::{Disks, System};


#[derive(Debug, Deserialize)]
#[serde(default)]
struct Config {
    logo: LogoConfig,
    display: DisplayConfig,
    modules: Vec<ModuleConfig>,
    color: bool,
    nerd_font: bool,
    image_protocol: String,
}
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum LogoConfig {
    Name(String),
    Options(LogoOptions),
}
#[derive(Debug, Deserialize)]
#[serde(default)]
struct LogoOptions {
    source: String,
    height: u32,
    padding: LogoPadding,
}
#[derive(Debug, Deserialize)]
#[serde(default)]
struct LogoPadding { top: usize, right: usize }
#[derive(Debug, Deserialize)]
#[serde(default)]
struct DisplayConfig {
    separator: String,
    key: KeyConfig,
    color: DisplayColors,
}
#[derive(Debug, Deserialize)]
#[serde(default)]
struct KeyConfig { width: usize }
#[derive(Debug, Deserialize)]
#[serde(default)]
struct DisplayColors { keys: String, title: String }
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ModuleConfig {
    Name(String),
    Options(ModuleOptions),
}
#[derive(Debug, Deserialize)]
#[serde(default)]
struct ModuleOptions {
    #[serde(rename = "type")]
    kind: String,
    key: String,
    format: String,
    text: String,
}
impl Default for LogoConfig {
    fn default() -> Self { Self::Options(LogoOptions::default()) }
}
impl Default for LogoOptions {
    fn default() -> Self {
        Self { source: "matsurika".into(), height: 24, padding: LogoPadding::default() }
    }
}
impl Default for LogoPadding {
    fn default() -> Self { Self { top: 0, right: 3 } }
}
impl Default for DisplayConfig {
    fn default() -> Self {
        Self { separator: "  ➜  ".into(), key: KeyConfig::default(), color: DisplayColors::default() }
    }
}
impl Default for KeyConfig { fn default() -> Self { Self { width: 14 } } }
impl Default for DisplayColors { fn default() -> Self { Self { keys: "white".into(), title: "white".into() } } }
impl Default for ModuleOptions {
    fn default() -> Self { Self { kind: String::new(), key: String::new(), format: String::new(), text: String::new() } }
}
impl Default for ModuleConfig {
    fn default() -> Self { Self::Name(String::new()) }
}
impl ModuleConfig {
    fn kind(&self) -> &str { match self { Self::Name(s) => s, Self::Options(o) => &o.kind } }
    fn key(&self) -> &str { match self { Self::Name(_) => "", Self::Options(o) => &o.key } }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            logo: LogoConfig::default(), display: DisplayConfig::default(),
            modules: default_modules(), color: true, nerd_font: true,
            image_protocol: "auto".into(),
        }
    }
}
fn default_modules() -> Vec<ModuleConfig> {
    ["title", "separator", "os", "kernel", "packages", "shell", "wm", "terminal",
     "break", "media", "battery", "uptime", "age", "break", "cpu", "gpu", "storage", "memory", "break", "colors"]
        .iter().map(|kind| ModuleConfig::Name((*kind).into())).collect()
}
impl Config {
    fn logo_source(&self) -> &str {
        match &self.logo { LogoConfig::Name(s) => s, LogoConfig::Options(o) => &o.source }
    }
    fn logo_height(&self) -> u32 {
        match &self.logo { LogoConfig::Name(_) => 24, LogoConfig::Options(o) => o.height.max(1) }
    }
    fn logo_padding(&self) -> (usize, usize) {
        match &self.logo { LogoConfig::Name(_) => (0, 3), LogoConfig::Options(o) => (o.padding.top, o.padding.right) }
    }
}
#[derive(Default)]
struct Info {
    user_host: String,
    os: String,
    host: String,
    kernel: String,
    uptime: String,
    packages: String,
    cpu: String,
    gpu: String,
    storage: String,
    battery: Option<String>,
    memory: String,
    shell: String,
    wm: String,
    terminal: String,
    media: String,
    age: String,
}

const GENTOO: &str = r#"
       _____
      /\    \
     /  \____\
    /   /    /
   /   /____/
  /         \
 /___________\
"#;

const ARCH: &str = r#"
       /\
      /  \
     /\   \
    /      \
   /   ,,   \
  /   |  |   \
 /_-''    ''-_\
"#;

const DEBIAN: &str = r#"
      .----.
   .-'  _   '-.
  /   .' '.    \
 |   /  _  \   |
  \  \ (_) /  /
   '-._____.-'
"#;

const UBUNTU: &str = r#"
       .---.
    .-'     '-.
   /  .--. .-. \
  |  /   / /  |
   \  '--' /  /
    '-.__.-'
"#;

const FEDORA: &str = r#"
      ______
     / ____/
    / /_ __
   / __// /
  /_/  /_/
"#;

const NIXOS: &str = r#"
      .---.
     /     \
    |  NIX  |
     \     /
      '---'
"#;

const LINUX: &str = r#"
   .--.
  |o_o |
  |:_/ |
 //   \ \
(|     | )
/'\_   _/`\
\___)=(___/
"#;

const MATSURIKA_TEXT: &str = r#"
  Matsurika
  Fetch
"#;

fn read_config() -> Config {
    let Some(config_dir) = dirs::config_dir() else { return Config::default() };
    let path = config_dir.join("mfetch/config.jsonc");
    match fs::read_to_string(path) {
        Ok(raw) => json5::from_str(&raw).unwrap_or_else(|error| {
            eprintln!("matsurikafetch: config parse error: {error}; using defaults");
            Config::default()
        }),
        Err(_) => Config::default(),
    }
}

fn os_release() -> (String, String) {
    let raw = fs::read_to_string("/etc/os-release").unwrap_or_default();
    let mut pretty = String::from("Linux");
    let mut id = String::from("linux");

    for line in raw.lines() {
        if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
            pretty = value.trim_matches('"').to_string();
        } else if let Some(value) = line.strip_prefix("ID=") {
            id = value.trim_matches('"').to_lowercase();
        }
    }
    (pretty, id)
}

fn package_count() -> String {
    // Gentoo's package database is checked first, since this is the target setup.
    let gentoo = Path::new("/var/db/pkg");
    if gentoo.is_dir() {
        let count = fs::read_dir(gentoo)
            .ok()
            .into_iter()
            .flatten()
            .flatten()
            .filter(|entry| entry.path().is_dir())
            .map(|entry| {
                fs::read_dir(entry.path())
                    .ok()
                    .map(|entries| entries.count())
                    .unwrap_or(0)
            })
            .sum::<usize>();
        return count.to_string();
    }

    let pacman = Path::new("/var/lib/pacman/local");
    if pacman.is_dir() {
        return fs::read_dir(pacman)
            .ok()
            .map(|entries| entries.count())
            .unwrap_or(0)
            .to_string();
    }

    if let Ok(status) = fs::read_to_string("/var/lib/dpkg/status") {
        let count = status
            .split("\n\n")
            .filter(|package| {
                package
                    .lines()
                    .any(|line| line.starts_with("Status: install ok installed"))
            })
            .count();
        return count.to_string();
    }

    if Path::new("/var/lib/rpm").exists() {
        return "rpm db".into();
    }
    "unknown".into()
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

fn format_bytes(bytes: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    if bytes as f64 >= GIB {
        format!("{:.0}G", bytes as f64 / GIB)
    } else {
        format!("{:.0}M", bytes as f64 / MIB)
    }
}

fn host_model() -> String {
    let base = Path::new("/sys/devices/virtual/dmi/id");
    let product = fs::read_to_string(base.join("product_name"))
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    let version = fs::read_to_string(base.join("product_version"))
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    match (product.is_empty(), version.is_empty()) {
        (false, false) if product != version => format!("{product} {version}"),
        (false, _) => product,
        _ => System::name().unwrap_or_else(|| "unknown".into()),
    }
}

fn cpu_info(sys: &System) -> String {
    let brand = sys
        .cpus()
        .first()
        .map(|cpu| cpu.brand().trim())
        .filter(|brand| !brand.is_empty())
        .unwrap_or("Unknown CPU");
    let threads = sys.cpus().len();
    let usage = sys.global_cpu_usage();
    if threads > 0 {
        format!("{brand} ({threads} threads, {usage:.0}%)")
    } else {
        brand.to_string()
    }
}

fn gpu_info() -> String {
    // nvidia-smi gives a much more useful model name when the NVIDIA driver exists.
    if let Ok(output) = Command::new("nvidia-smi")
        .args(["--query-gpu=name", "--format=csv,noheader"])
        .output()
    {
        if output.status.success() {
            // Keep the decoded output alive while `names` borrows its lines.
            // Calling `lines()` directly on `from_utf8_lossy(...)` would leave
            // the collected string slices borrowing a temporary value.
            let stdout = String::from_utf8_lossy(&output.stdout);
            let names = stdout
                .lines()
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .collect::<Vec<_>>();
            if !names.is_empty() {
                return names.join(" + ");
            }
        }
    }

    let mut detected = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/drm") else {
        return "not detected".into();
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        // Only card devices, not connectors such as card0-DP-1.
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let device = entry.path().join("device");
        let vendor = fs::read_to_string(device.join("vendor"))
            .unwrap_or_default()
            .trim()
            .to_lowercase();
        if vendor.is_empty() {
            continue;
        }
        let driver = fs::read_link(device.join("driver"))
            .ok()
            .and_then(|path| path.file_name().map(|part| part.to_string_lossy().into_owned()))
            .unwrap_or_default();
        let vendor_name = match vendor.as_str() {
            "0x10de" => "NVIDIA",
            "0x1002" => "AMD Radeon",
            "0x8086" => "Intel Graphics",
            "0x1af4" => "Virtio GPU",
            _ => "GPU",
        };
        let pci_slot = fs::read_to_string(device.join("uevent"))
            .ok()
            .and_then(|contents| {
                contents.lines().find_map(|line| line.strip_prefix("PCI_SLOT_NAME="))
                    .map(str::to_string)
            });
        let pci_model = pci_slot.and_then(|slot| {
            Command::new("lspci")
                .args(["-s", slot.as_str()])
                .output()
                .ok()
                .filter(|output| output.status.success())
                .and_then(|output| {
                    let line = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    line.split_once("controller: ")
                        .map(|(_, model)| model.trim().to_string())
                        .or_else(|| line.split_once("Display controller: ")
                            .map(|(_, model)| model.trim().to_string()))
                })
        });
        let model = pci_model.unwrap_or_else(|| vendor_name.to_string());
        let label = if driver.is_empty() {
            model
        } else {
            format!("{model} ({driver})")
        };
        if !detected.contains(&label) {
            detected.push(label);
        }
    }

    if detected.is_empty() {
        "not detected".into()
    } else {
        detected.join(" + ")
    }
}

fn storage_info() -> String {
    let disks = Disks::new_with_refreshed_list();
    // Show the filesystem backing `/` to avoid filling the fetch output with
    // temporary mounts, containers and removable devices.
    if let Some(root) = disks.list().iter().find(|disk| disk.mount_point() == Path::new("/")) {
        let total = root.total_space();
        let used = total.saturating_sub(root.available_space());
        return format!("{} / {}", format_bytes(used), format_bytes(total));
    }
    "not available".into()
}

fn battery_info() -> Option<String> {
    let supplies = fs::read_dir("/sys/class/power_supply").ok()?;
    for supply in supplies.flatten() {
        let path = supply.path();
        let kind = fs::read_to_string(path.join("type")).unwrap_or_default();
        if kind.trim() != "Battery" {
            continue;
        }
        let capacity = match fs::read_to_string(path.join("capacity"))
            .ok()
            .and_then(|value| value.trim().parse::<u8>().ok())
        {
            Some(value) => value,
            None => continue,
        };
        let status = fs::read_to_string(path.join("status"))
            .unwrap_or_else(|_| "Unknown".into())
            .trim()
            .to_string();
        return Some(format!("{capacity}% ({status})"));
    }
    None
}

fn collect() -> Info {
    let mut sys = System::new_all();
    sys.refresh_all();
    // CPU utilisation is calculated from two samples, so take a second sample.
    thread::sleep(Duration::from_millis(200));
    sys.refresh_cpu_usage();

    let (os, _) = os_release();
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "user".into());
    let hostname = System::host_name().unwrap_or_else(|| "unknown".into());
    let kernel = System::kernel_version().unwrap_or_else(|| "unknown".into());
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();

    Info {
        user_host: format!("{user}@{hostname}"),
        os,
        host: host_model(),
        kernel,
        uptime: format_uptime(System::uptime()),
        packages: package_count(),
        cpu: cpu_info(&sys),
        gpu: gpu_info(),
        storage: storage_info(),
        battery: battery_info(),
        memory: format!("{} / {}", format_bytes(used_memory), format_bytes(total_memory)),
        shell: env::var("SHELL").ok().and_then(|s| Path::new(&s).file_name().map(|x| x.to_string_lossy().into_owned())).unwrap_or_else(|| "unknown".into()),
        wm: env::var("XDG_CURRENT_DESKTOP").or_else(|_| env::var("DESKTOP_SESSION")).unwrap_or_else(|_| "unknown".into()),
        terminal: env::var("TERM_PROGRAM").or_else(|_| env::var("TERM")).unwrap_or_else(|_| "unknown".into()),
        media: "Nothing playing".into(),
        age: fs::metadata("/").ok().and_then(|m| m.created().ok()).map(|t| format!("{} days", t.elapsed().unwrap_or_default().as_secs()/86400)).unwrap_or_else(|| "unknown".into()),
    }
}

fn distro_logo(distro: &str) -> &'static str {
    match distro {
        "gentoo" => GENTOO,
        "arch" | "archlinux" | "endeavouros" | "manjaro" => ARCH,
        "debian" | "devuan" => DEBIAN,
        "ubuntu" | "linuxmint" | "pop" => UBUNTU,
        "fedora" => FEDORA,
        "nixos" => NIXOS,
        _ => LINUX,
    }
}

fn matsurika_image() -> Option<image::DynamicImage> {
    let decoded = image::load_from_memory(include_bytes!("../assets/matsurika.png")).ok()?;
    let rgba = decoded.to_rgba8();

    // Remove only fully transparent margins. This keeps the artwork's original
    // pixels and aspect ratio, while preventing transparent padding from
    // pushing the information column unnecessarily far to the right.
    let mut min_x = rgba.width();
    let mut min_y = rgba.height();
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found = false;

    for (x, y, pixel) in rgba.enumerate_pixels() {
        if pixel[3] >= 8 {
            found = true;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }

    if !found {
        return None;
    }

    let cropped = image::imageops::crop_imm(
        &rgba,
        min_x,
        min_y,
        max_x - min_x + 1,
        max_y - min_y + 1,
    )
    .to_image();

    Some(image::DynamicImage::ImageRgba8(cropped))
}

fn write_info_panel(right: &[String], logo_width: u32, logo_height: u32, top: usize, right_padding: usize) -> bool {
    // viuer restores the cursor to the command's starting position. Save that
    // position and place each information line relative to it, not row 1 of
    // the entire terminal (which would overwrite the prompt/history).
    let text_column = logo_width as usize + right_padding;
    let last_row = (logo_height as usize).max(right.len());
    let mut stdout = io::stdout();
    if stdout.write_all(b"\x1b[s").is_err() { return false; }

    for (row, line) in right.iter().enumerate() {
        let down = top + row;
        let output = format!("\x1b[u\x1b[{}B\x1b[{}C{}\x1b[K", down, text_column, line);
        if stdout.write_all(output.as_bytes()).is_err() { return false; }
    }
    let cursor = format!("\x1b[u\x1b[{}B\r\n", last_row + 1);
    stdout.write_all(cursor.as_bytes()).is_ok() && stdout.flush().is_ok()
}

fn render_viuer_with_panel(
    right: &[String],
    config: &Config,
    color: bool,
    use_kitty: bool,
    use_iterm: bool,
) -> bool {
    if !io::stdout().is_terminal() || !io::stdin().is_terminal() {
        return false;
    }

    let Some(image) = matsurika_image() else {
        return false;
    };

    let image_config = viuer::Config {
        x: 0,
        y: config.logo_padding().0 as i16,
        absolute_offset: false,
        restore_cursor: true,
        transparent: true,
        premultiplied_alpha: false,
        truecolor: color,
        width: None,
        height: Some(config.logo_height().max(1)),
        // Let viuer select the terminal's native protocol (Kitty, iTerm2,
        // or Sixel) and retain its built-in half-block fallback.
        use_sixel: true,
        use_iterm,
        use_kitty,
        ..Default::default()
    };

    let Ok((logo_width, logo_height)) = viuer::print(&image, &image_config) else {
        return false;
    };
    let (top, right_padding) = config.logo_padding();
    write_info_panel(right, logo_width, logo_height, top, right_padding)
}

fn render_matsurika_with_panel(right: &[String], config: &Config, color: bool) -> bool {
    if !io::stdout().is_terminal() || !io::stdin().is_terminal() { return false; }
    match config.image_protocol.to_ascii_lowercase().as_str() {
        "halfblocks" => render_viuer_with_panel(right, config, color, false, false),
        "kitty" => render_viuer_with_panel(right, config, color, true, false),
        "iterm" => render_viuer_with_panel(right, config, color, false, true),
        // `auto` delegates capability detection to viuer instead of guessing
        // based on TERM or sending a protocol the terminal may not support.
        "auto" | "sixel" => render_viuer_with_panel(right, config, color, true, true),
        _ => render_viuer_with_panel(right, config, color, true, true),
    }
}

fn ascii_lines(art: &'static str) -> Vec<String> {
    art.lines().map(str::to_string).collect()
}

fn distro_icon(distro: &str) -> &'static str {
    match distro {
        "gentoo" => "",
        "arch" | "archlinux" | "endeavouros" | "manjaro" => "",
        "debian" | "devuan" => "",
        "ubuntu" | "linuxmint" | "pop" => "",
        "fedora" => "",
        "nixos" => "",
        _ => "",
    }
}


fn main() {
    let config = read_config();
    let info = collect();
    let (_, distro_id) = os_release();
    let stdout_is_terminal = io::stdout().is_terminal();
    let use_color = config.color && stdout_is_terminal;
    let use_image = !config.logo_source().eq_ignore_ascii_case("auto");

    let cyan = if use_color { "\x1b[96m" } else { "" };
    let pink = if use_color { "\x1b[95m" } else { "" };
    let reset = if use_color { "\x1b[0m" } else { "" };
    let bold = if use_color { "\x1b[1m" } else { "" };
    let key_width = config.display.key.width;
    let key_color = if use_color && config.display.color.keys.eq_ignore_ascii_case("white") { "\x1b[97m" } else { cyan };
    let title_color = if use_color && config.display.color.title.eq_ignore_ascii_case("white") { "\x1b[97m" } else { pink };

    let module_value = |kind: &str| -> String {
        match kind {
            "title" => format!("{bold}{title_color}{}{reset}", info.user_host),
            "separator" => "────────────────────────".into(),
            "break" | "colors" => String::new(),
            "os" => info.os.clone(),
            "kernel" => info.kernel.clone(),
            "packages" | "pkgs" => info.packages.clone(),
            "shell" => info.shell.clone(),
            "wm" => info.wm.clone(),
            "terminal" => info.terminal.clone(),
            "media" => info.media.clone(),
            "battery" => info.battery.clone().unwrap_or_else(|| "not detected".into()),
            "uptime" => info.uptime.clone(),
            "age" => info.age.clone(),
            "cpu" => info.cpu.clone(),
            "gpu" => info.gpu.clone(),
            "storage" => info.storage.clone(),
            "memory" => info.memory.clone(),
            _ => String::new(),
        }
    };
    let mut right = Vec::new();
    for module in &config.modules {
        let kind = module.kind().to_ascii_lowercase();
        if kind == "break" {
            right.push(String::new());
            continue;
        }
        if kind == "separator" {
            right.push(module_value(&kind));
            continue;
        }
        if kind == "colors" {
            let swatches = if use_color {
                "\x1b[40m  \x1b[41m  \x1b[42m  \x1b[43m  \x1b[44m  \x1b[45m  \x1b[46m  \x1b[47m  \x1b[0m".to_string()
            } else { String::new() };
            right.push(swatches);
            continue;
        }
        let value = module_value(&kind);
        if kind == "title" {
            right.push(value);
            continue;
        }
        if kind == "command" { continue; } // Arbitrary shell commands are intentionally not executed.
        let key = if module.key().is_empty() {
            match kind.as_str() {
                "os"=>"  os", "kernel"=>"⚙  ker", "packages"=>"  pkg",
                "shell"=>"  sh", "wm"=>"  wm", "terminal"=>"  term",
                "media"=>"  med", "battery"=>"  bat", "uptime"=>"⏱  up",
                "age"=>"  age", "cpu"=>"  cpu", "gpu"=>"  gpu",
                "storage"=>"  disk", "memory"=>"  mem", _=>kind.as_str()
            }.to_string()
        } else { module.key().to_string() };
        let key = if config.nerd_font { key } else { key.chars().filter(|c| c.is_ascii_alphanumeric() || *c == ' ').collect() };
        let key = format!("{key:<width$}", width=key_width);
        let line = format!("{key_color}{key}{reset}{}{value}", config.display.separator);
        right.push(line);
    }

    if use_image && render_matsurika_with_panel(&right, &config, use_color) { return; }

    let art = if config.logo_source().eq_ignore_ascii_case("auto") { distro_logo(&distro_id) } else { MATSURIKA_TEXT };
    let art_lines = ascii_lines(art);
    let height = art_lines.len().max(right.len());
    let (top, right_padding) = config.logo_padding();
    for _ in 0..top { println!(); }
    for index in 0..height {
        let left = art_lines.get(index).map(String::as_str).unwrap_or("");
        let right_line = right.get(index).map(String::as_str).unwrap_or("");
        println!("{left:<22}{}{right_line}", " ".repeat(right_padding));
    }
}
