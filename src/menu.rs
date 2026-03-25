//! Command menu rendered as per-section "cards".
//!
//! Each section (DEVICE, PACKAGES, …) appears as:
//!
//!   SECTION TITLE ─────────────────────────────
//!   ╭──────────────────────────────────────────╮
//!   │   icon  Label                            │
//!   │ ▶ icon  Selected label                   │  ← highlighted
//!   ╰──────────────────────────────────────────╯
//!
//! The full stack of cards scrolls vertically to keep the selected item visible.

use crate::adb::{AdbCommand, PackageFilter};
use crate::fastboot::FastbootCommand;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::Widget,
};

// ── Command wrapper ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum MenuCommand {
    Adb(AdbCommand),
    Fastboot(FastbootCommand),
    OpenLogcat,
    OpenDevMode,
}

// ── Entry types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum MenuEntry {
    Section(&'static str),
    Item {
        label: &'static str,
        description: &'static str,
        command: MenuCommand,
        danger: bool,
    },
    Spacer,
}

impl MenuEntry {
    pub fn is_selectable(&self) -> bool {
        matches!(self, MenuEntry::Item { .. })
    }
}

// ── Menu state ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct Menu {
    pub entries: Vec<MenuEntry>,
    /// Full-list index of the currently selected `Item`.
    pub selected: usize,
    pub tick_count: u64,
    /// How many virtual rows are hidden above the top of the visible area.
    pub scroll_offset: u16,
    /// Visible height recorded during the last render — used for scroll math.
    last_height: u16,
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

impl Menu {
    pub fn new() -> Self {
        let entries = build_entries();
        let first = entries.iter().position(|e| e.is_selectable()).unwrap_or(0);
        Self {
            entries,
            selected: first,
            tick_count: 0,
            scroll_offset: 0,
            last_height: 20,
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
    }

    pub fn next(&mut self) {
        let len = self.entries.len();
        for i in (self.selected + 1)..len {
            if self.entries[i].is_selectable() {
                return self.commit(i);
            }
        }
        for i in 0..self.selected {
            if self.entries[i].is_selectable() {
                return self.commit(i);
            }
        }
    }

    pub fn previous(&mut self) {
        for i in (0..self.selected).rev() {
            if self.entries[i].is_selectable() {
                return self.commit(i);
            }
        }
        for i in (0..self.entries.len()).rev() {
            if self.entries[i].is_selectable() {
                return self.commit(i);
            }
        }
    }

    /// Jump to the first item of the next section (wraps around).
    pub fn next_section(&mut self) {
        let groups = build_groups(&self.entries);
        let cur = groups
            .iter()
            .position(|g| g.item_indices.contains(&self.selected))
            .unwrap_or(0);
        let next = (cur + 1) % groups.len();
        if let Some(&first) = groups[next].item_indices.first() {
            self.commit(first);
        }
    }

    /// Jump to the first item of the previous section (wraps around).
    pub fn previous_section(&mut self) {
        let groups = build_groups(&self.entries);
        let cur = groups
            .iter()
            .position(|g| g.item_indices.contains(&self.selected))
            .unwrap_or(0);
        let prev = if cur == 0 { groups.len() - 1 } else { cur - 1 };
        if let Some(&first) = groups[prev].item_indices.first() {
            self.commit(first);
        }
    }

    fn commit(&mut self, idx: usize) {
        self.selected = idx;
        self.update_scroll();
    }

    pub fn get_selected_command(&self) -> MenuCommand {
        match &self.entries[self.selected] {
            MenuEntry::Item { command, .. } => command.clone(),
            _ => MenuCommand::Adb(AdbCommand::ListDevices),
        }
    }

    pub fn get_selected_description(&self) -> &str {
        match &self.entries[self.selected] {
            MenuEntry::Item { description, .. } => description,
            _ => "",
        }
    }

    pub fn get_selected_label(&self) -> &str {
        match &self.entries[self.selected] {
            MenuEntry::Item { label, .. } => label,
            _ => "",
        }
    }

    /// Adjust `scroll_offset` so the selected item stays inside the viewport.
    fn update_scroll(&mut self) {
        let h = self.last_height as i32;
        if h <= 2 {
            return;
        }

        let groups = build_groups(&self.entries);
        let mut vy: i32 = 0; // virtual row counter

        for group in &groups {
            let section_start = vy;
            vy += 1; // header row
            vy += 1; // top border row

            for &idx in &group.item_indices {
                if idx == self.selected {
                    let scroll = self.scroll_offset as i32;
                    if vy < scroll {
                        // Item is above — scroll up to reveal the section header too
                        self.scroll_offset = section_start.max(0) as u16;
                    } else if vy >= scroll + h {
                        // Item is below — scroll down so item sits at the bottom
                        self.scroll_offset = (vy - h + 1) as u16;
                    }
                    return;
                }
                vy += 1;
            }

            vy += 1; // bottom border row
            vy += 1; // spacer row
        }
    }
}

// ── Section groups ────────────────────────────────────────────────────────────

struct SectionGroup {
    title: &'static str,
    item_indices: Vec<usize>, // indices into Menu::entries
}

fn build_groups(entries: &[MenuEntry]) -> Vec<SectionGroup> {
    let mut groups: Vec<SectionGroup> = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        match entry {
            MenuEntry::Section(t) => groups.push(SectionGroup {
                title: t,
                item_indices: Vec::new(),
            }),
            MenuEntry::Item { .. } => {
                if let Some(g) = groups.last_mut() {
                    g.item_indices.push(i);
                }
            }
            MenuEntry::Spacer => {}
        }
    }
    groups
}

// ── Widget ────────────────────────────────────────────────────────────────────

impl Widget for &mut Menu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.last_height = area.height;

        let groups = build_groups(&self.entries);
        let sel = self.selected;
        let scroll = self.scroll_offset as i32;

        let col_active = Color::Rgb(50, 170, 50);
        let col_inactive = Color::Rgb(40, 65, 40);

        let mut vy: i32 = 0;

        for group in &groups {
            let n = group.item_indices.len() as i32;
            let group_h = 1 + 1 + n + 1 + 1; // header + top + items + bot + spacer

            // skip groups entirely above the viewport
            if vy + group_h <= scroll {
                vy += group_h;
                continue;
            }
            // stop once we're entirely below
            if vy - scroll >= area.height as i32 {
                break;
            }

            let is_active = group.item_indices.contains(&sel);
            let bdr_color = if is_active { col_active } else { col_inactive };

            // section header
            let sy = vy - scroll;
            if sy >= 0 && sy < area.height as i32 {
                draw_section_header(
                    group.title,
                    area.x,
                    area.y + sy as u16,
                    area.width,
                    buf,
                    is_active,
                );
            }
            vy += 1;

            // top border
            let sy = vy - scroll;
            if sy >= 0 && sy < area.height as i32 {
                draw_border_top(area.x, area.y + sy as u16, area.width, buf, bdr_color);
            }
            vy += 1;

            // items
            for &idx in &group.item_indices {
                let sy = vy - scroll;
                if sy >= 0 && sy < area.height as i32 {
                    if let MenuEntry::Item { label, danger, .. } = &self.entries[idx] {
                        draw_item(
                            area.x,
                            area.y + sy as u16,
                            area.width,
                            buf,
                            label,
                            idx == sel,
                            *danger,
                            bdr_color,
                        );
                    }
                }
                vy += 1;
            }

            // bottom border
            let sy = vy - scroll;
            if sy >= 0 && sy < area.height as i32 {
                draw_border_bottom(area.x, area.y + sy as u16, area.width, buf, bdr_color);
            }
            vy += 1;

            vy += 1; // spacer between cards
        }
    }
}

// ── Drawing helpers ───────────────────────────────────────────────────────────

fn draw_section_header(title: &str, x: u16, y: u16, width: u16, buf: &mut Buffer, active: bool) {
    let title_style = if active {
        Style::default()
            .fg(Color::Rgb(130, 210, 130))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Rgb(120, 120, 120))
            .add_modifier(Modifier::BOLD)
    };
    let rule_style = if active {
        Style::default().fg(Color::Rgb(45, 90, 45))
    } else {
        Style::default().fg(Color::Rgb(45, 45, 45))
    };

    let title_w = Span::raw(title).width();
    buf.set_string(x, y, title, title_style);

    let rule_x = x + title_w as u16 + 1;
    if rule_x < x + width {
        let rule = "\u{2500}".repeat((x + width - rule_x) as usize); // "─"
        buf.set_string(rule_x, y, &rule, rule_style);
    }
}

fn draw_border_top(x: u16, y: u16, width: u16, buf: &mut Buffer, color: Color) {
    let style = Style::default().fg(color);
    let inner = width.saturating_sub(2) as usize;
    set_cell(buf, x, y, '\u{256d}', style); // ╭
    for i in 0..inner {
        set_cell(buf, x + 1 + i as u16, y, '\u{2500}', style); // ─
    }
    set_cell(buf, x + width - 1, y, '\u{256e}', style); // ╮
}

fn draw_border_bottom(x: u16, y: u16, width: u16, buf: &mut Buffer, color: Color) {
    let style = Style::default().fg(color);
    let inner = width.saturating_sub(2) as usize;
    set_cell(buf, x, y, '\u{2570}', style); // ╰
    for i in 0..inner {
        set_cell(buf, x + 1 + i as u16, y, '\u{2500}', style); // ─
    }
    set_cell(buf, x + width - 1, y, '\u{256f}', style); // ╯
}

#[allow(clippy::too_many_arguments)]
fn draw_item(
    x: u16,
    y: u16,
    width: u16,
    buf: &mut Buffer,
    label: &str,
    is_selected: bool,
    is_danger: bool,
    bdr_color: Color,
) {
    let inner_w = width.saturating_sub(2);
    let bdr = Style::default().fg(bdr_color);

    // Left border
    set_cell(buf, x, y, '\u{2502}', bdr); // │

    // Content style + prefix
    let (prefix, style) = if is_selected {
        let bg = if is_danger {
            Color::Rgb(170, 70, 20)
        } else {
            Color::Rgb(50, 170, 50)
        };
        (
            " \u{25b6} ", // ▶
            Style::default()
                .fg(Color::Rgb(10, 10, 10))
                .bg(bg)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        let fg = if is_danger {
            Color::Rgb(200, 110, 50)
        } else {
            Color::Rgb(210, 210, 210)
        };
        ("   ", Style::default().fg(fg))
    };

    // For selected rows: flood-fill the inner width with the highlight background
    // so the row looks fully highlighted even past the text.
    if is_selected {
        let bg = if is_danger {
            Color::Rgb(170, 70, 20)
        } else {
            Color::Rgb(50, 170, 50)
        };
        for col in 0..inner_w {
            if let Some(cell) = buf.cell_mut((x + 1 + col, y)) {
                cell.set_char(' ');
                cell.set_bg(bg);
                cell.set_fg(Color::Rgb(10, 10, 10));
            }
        }
    }

    // Write prefix + label over the filled background
    let content = format!("{}{}", prefix, label);
    buf.set_string(x + 1, y, &content, style);

    // Right border — written last so it always wins
    set_cell(buf, x + width - 1, y, '\u{2502}', bdr); // │
}

#[inline]
fn set_cell(buf: &mut Buffer, x: u16, y: u16, ch: char, style: Style) {
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_char(ch);
        cell.set_style(style);
    }
}

// ── Entry list ────────────────────────────────────────────────────────────────

fn item(
    label: &'static str,
    description: &'static str,
    command: MenuCommand,
    danger: bool,
) -> MenuEntry {
    MenuEntry::Item {
        label,
        description,
        command,
        danger,
    }
}
fn adb(cmd: AdbCommand) -> MenuCommand {
    MenuCommand::Adb(cmd)
}
fn fb(cmd: FastbootCommand) -> MenuCommand {
    MenuCommand::Fastboot(cmd)
}
fn sh(command: &str) -> MenuCommand {
    adb(AdbCommand::Shell {
        command: command.into(),
    })
}

fn build_entries() -> Vec<MenuEntry> {
    vec![
        // ── Dev Tools ───────────────────────────────────────────────────────
        MenuEntry::Section("DEV TOOLS"),
        item("🛠   Dev Mode",
             "Open the developer workstation — build, run, edit, and logcat in one view",
             MenuCommand::OpenDevMode,
             false),
        MenuEntry::Spacer,

        // ── Device ──────────────────────────────────────────────────────────
        MenuEntry::Section("DEVICE"),
        item("📱  List Devices",
             "Show all connected Android devices and their connection status",
             adb(AdbCommand::ListDevices), false),
        item("🔍  Device Model",
             "Show device model, brand and product name",
             sh("getprop | grep -E 'ro.product.model|ro.product.brand|ro.product.name'"), false),
        item("🗒   Android Version",
             "Show Android version, build ID and security patch level",
             sh("getprop | grep -E 'ro.build.version|ro.build.id|ro.build.date'"), false),
        item("🔢  ADB Version",
             "Display the current ADB server version",
             adb(AdbCommand::GetAdbVersion), false),
        MenuEntry::Spacer,

        // ── Packages ────────────────────────────────────────────────────────
        MenuEntry::Section("PACKAGES"),
        item("📦  All Packages",
             "List every installed package — system and user",
             adb(AdbCommand::ListPackages { include_path: false, filter: PackageFilter::All }), false),
        item("👤  User Packages",
             "List only third-party user-installed packages",
             adb(AdbCommand::ListPackages { include_path: false, filter: PackageFilter::User }), false),
        item("⚙   System Packages",
             "List only built-in system packages",
             adb(AdbCommand::ListPackages { include_path: false, filter: PackageFilter::System }), false),
        item("📁  Packages with Paths",
             "List all packages together with their APK file paths",
             adb(AdbCommand::ListPackages { include_path: true,  filter: PackageFilter::All }), false),
        MenuEntry::Spacer,

        // ── System ──────────────────────────────────────────────────────────
        MenuEntry::Section("SYSTEM"),
        item("📺  Live Logcat",
             "Open the live logcat viewer with real-time streaming, search, and level filters",
             MenuCommand::OpenLogcat,
             false),
        item("🔋  Battery Status",
             "Show detailed battery info — level, health and temperature",
             adb(AdbCommand::GetBatteryInfo), false),
        item("💾  Memory Usage",
             "Show total, available and used RAM",
             adb(AdbCommand::GetMemoryInfo), false),
        item("📊  CPU Info",
             "Display CPU architecture, core count and clock frequency",
             adb(AdbCommand::GetCpuInfo), false),
        item("🏃  Running Processes",
             "List all currently running processes",
             adb(AdbCommand::ListProcesses), false),
        item("📜  System Log",
             "View the last 100 lines of the system log",
             adb(AdbCommand::GetSystemLog { lines: 100 }), false),
        item("🚨  Error Log",
             "Show only error-level log entries",
             sh("logcat -d *:E"), false),
        item("🔧  System Services",
             "List all registered Android system services",
             sh("service list"), false),
        item("🏷   Device Properties",
             "Dump all Android system properties via getprop",
             adb(AdbCommand::GetDeviceProperties), false),
        MenuEntry::Spacer,

        // ── Network ─────────────────────────────────────────────────────────
        MenuEntry::Section("NETWORK"),
        item("🌐  Network Status",
             "Show network connectivity and interface status",
             adb(AdbCommand::GetNetworkInfo), false),
        item("📶  WiFi Info",
             "Display WiFi interface name and signal info",
             adb(AdbCommand::GetWifiStatus), false),
        item("🔗  IP Configuration",
             "Show IP addresses for all network interfaces",
             sh("ip addr show"), false),
        MenuEntry::Spacer,

        // ── Root Toolkit ─────────────────────────────────────────────────────
        MenuEntry::Section("ROOT TOOLKIT"),
        item("🔐  Root Status",
             "Check whether the device is rooted (requires ADB shell access)",
             sh("su -c 'id' 2>/dev/null && echo '' && echo 'Device is ROOTED' || echo 'Device is NOT ROOTED'"), false),
        item("🔒  SELinux Status",
             "Check current SELinux enforcement mode (Enforcing / Permissive)",
             sh("getenforce"), false),
        item("🪄  Magisk Status",
             "Check whether Magisk is installed and show version",
             sh("magisk --version 2>/dev/null || (ls /data/adb/magisk 2>/dev/null && echo 'Magisk files present') || echo 'Magisk not found'"), false),
        item("📋  Bootloader State",
             "Check bootloader lock state and verified boot status",
             sh("echo 'Verified boot:' && getprop ro.boot.verifiedbootstate; echo 'Secure:' && getprop ro.secure; echo 'Debuggable:' && getprop ro.debuggable"), false),
        item("⚠   SELinux Permissive",
             "Set SELinux to Permissive mode — reduces security. Requires root. ⚠ Dangerous",
             sh("su -c 'setenforce 0 && echo OK && getenforce'"), true),
        MenuEntry::Spacer,

        // ── Bootloader & Flash ───────────────────────────────────────────────
        MenuEntry::Section("BOOTLOADER & FLASH"),
        item("🔃  Reboot to Recovery",
             "Reboot device into recovery mode (e.g. TWRP) via ADB",
             sh("reboot recovery"), false),
        item("⚡  Reboot to Bootloader",
             "Reboot device into fastboot / bootloader mode via ADB",
             sh("reboot bootloader"), false),
        item("ℹ   Device Info  [fastboot]",
             "Retrieve all fastboot variables — requires device in bootloader mode",
             fb(FastbootCommand::GetVarAll), false),
        item("🔓  OEM Unlock  [fastboot]",
             "Unlock the bootloader to enable flashing  ⚠ WIPES ALL DATA on the device",
             fb(FastbootCommand::OemUnlock), true),
        item("🔒  OEM Lock  [fastboot]",
             "Re-lock the bootloader — device will verify boot integrity on startup",
             fb(FastbootCommand::OemLock), true),
        item("💣  Wipe Data  [fastboot]",
             "Factory reset: erase userdata + cache partitions  ⚠ ALL DATA PERMANENTLY LOST",
             fb(FastbootCommand::WipeData), true),
        MenuEntry::Spacer,

        // ── Actions ──────────────────────────────────────────────────────────
        MenuEntry::Section("ACTIONS"),
        item("📸  Take Screenshot",
             "Capture the screen and save to /sdcard/screenshot.png on the device",
             adb(AdbCommand::TakeScreenshot), false),
        item("📐  Screen Resolution",
             "Show physical screen size and pixel density",
             adb(AdbCommand::GetScreenResolution), false),
        item("🗑   Clear Logs",
             "Flush the entire Android log buffer",
             sh("logcat -c && echo 'Log buffer cleared'"), false),
        item("🔄  Reboot Device",
             "Reboot the connected device normally via ADB",
             sh("reboot"), false),
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_selected_is_item() {
        let menu = Menu::new();
        assert!(menu.entries[menu.selected].is_selectable());
    }

    #[test]
    fn test_next_skips_non_items() {
        let mut menu = Menu::new();
        let start = menu.selected;
        menu.next();
        assert!(menu.entries[menu.selected].is_selectable());
        assert_ne!(menu.selected, start);
    }

    #[test]
    fn test_previous_wraps_to_last() {
        let mut menu = Menu::new();
        menu.previous();
        assert!(menu.entries[menu.selected].is_selectable());
    }

    #[test]
    fn test_next_wraps_to_first() {
        let mut menu = Menu::new();
        for _ in 0..200 {
            menu.next();
        }
        menu.next();
        assert!(menu.entries[menu.selected].is_selectable());
    }

    #[test]
    fn test_get_selected_command_returns_first_item() {
        let menu = Menu::new();
        // First selectable item is now Dev Mode (DEV TOOLS section comes first)
        assert!(matches!(
            menu.get_selected_command(),
            MenuCommand::OpenDevMode
        ));
    }

    #[test]
    fn test_get_selected_description_not_empty() {
        let menu = Menu::new();
        assert!(!menu.get_selected_description().is_empty());
    }

    #[test]
    fn test_get_selected_label_not_empty() {
        let menu = Menu::new();
        assert!(!menu.get_selected_label().is_empty());
    }

    #[test]
    fn test_all_items_have_non_empty_description() {
        let entries = build_entries();
        for entry in &entries {
            if let MenuEntry::Item {
                description, label, ..
            } = entry
            {
                assert!(!description.is_empty(), "Empty description for: {}", label);
            }
        }
    }

    #[test]
    fn test_build_groups_covers_all_items() {
        let entries = build_entries();
        let groups = build_groups(&entries);
        let total_in_groups: usize = groups.iter().map(|g| g.item_indices.len()).sum();
        let total_items = entries.iter().filter(|e| e.is_selectable()).count();
        assert_eq!(total_in_groups, total_items);
    }

    #[test]
    fn test_scroll_offset_adjusts_on_navigation() {
        let mut menu = Menu::new();
        menu.last_height = 10;
        // Navigate far forward to trigger downward scroll
        for _ in 0..30 {
            menu.next();
        }
        assert!(menu.scroll_offset > 0, "scroll_offset should have advanced");
    }

    #[test]
    fn test_next_section_moves_to_different_section() {
        let mut menu = Menu::new();
        // Start at first item (DEVICE section)
        let initial = menu.selected;
        menu.next_section();
        // Should now be in a different section (first item of PACKAGES)
        assert!(menu.entries[menu.selected].is_selectable());
        assert_ne!(
            menu.selected, initial,
            "next_section should move to a new section"
        );

        let groups = build_groups(&menu.entries);
        let initial_group = groups
            .iter()
            .position(|g| g.item_indices.contains(&initial))
            .unwrap();
        let new_group = groups
            .iter()
            .position(|g| g.item_indices.contains(&menu.selected))
            .unwrap();
        assert_ne!(
            initial_group, new_group,
            "should be in a different group after next_section"
        );
    }

    #[test]
    fn test_next_section_always_lands_on_first_item_of_section() {
        let mut menu = Menu::new();
        let groups = build_groups(&menu.entries);

        for _ in 0..groups.len() {
            menu.next_section();
            let sel = menu.selected;
            // The selected item must be the *first* item in its group
            let group = groups
                .iter()
                .find(|g| g.item_indices.contains(&sel))
                .unwrap();
            assert_eq!(
                group.item_indices[0], sel,
                "next_section must land on the first item of the group"
            );
        }
    }

    #[test]
    fn test_next_section_wraps_to_first_section() {
        let mut menu = Menu::new();
        let groups = build_groups(&menu.entries);
        // Cycle through all sections — one more brings us back to the start
        for _ in 0..groups.len() {
            menu.next_section();
        }
        // After a full cycle we should be back at the first section's first item
        let first_section_first_item = groups[0].item_indices[0];
        assert_eq!(
            menu.selected, first_section_first_item,
            "next_section should wrap around"
        );
    }

    #[test]
    fn test_previous_section_moves_to_different_section() {
        let mut menu = Menu::new();
        let initial = menu.selected;
        menu.previous_section();
        assert!(menu.entries[menu.selected].is_selectable());
        assert_ne!(
            menu.selected, initial,
            "previous_section should move to a new section"
        );
    }

    #[test]
    fn test_previous_section_wraps_to_last_section() {
        let mut menu = Menu::new();
        let groups = build_groups(&menu.entries);
        // From the first section, going back should wrap to the last
        menu.previous_section();
        let last_section_first_item = groups.last().unwrap().item_indices[0];
        assert_eq!(
            menu.selected, last_section_first_item,
            "previous_section from first should wrap to last"
        );
    }

    #[test]
    fn test_next_and_previous_section_are_inverse() {
        let mut menu = Menu::new();
        let start = menu.selected;
        menu.next_section();
        menu.previous_section();
        assert_eq!(
            menu.selected, start,
            "next_section + previous_section should return to start"
        );
    }
}
