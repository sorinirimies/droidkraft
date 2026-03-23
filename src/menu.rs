// Removed unused import: get_menu_border_color
use crate::adb::{AdbCommand, PackageFilter};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph, Widget},
};

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub description: String,
    pub command: AdbCommand,
    pub children: Vec<MenuChild>,
}

#[derive(Debug, Clone)]
pub struct MenuChild {
    pub label: String,
    pub description: String,
    pub command: AdbCommand,
}

#[derive(Debug)]
pub struct Menu {
    pub items: Vec<MenuItem>,
    pub state: ListState,
    pub selected: usize,
    pub tick_count: u64,
    pub last_selected: usize,
    pub position_change_boost: u64,
    pub in_child_mode: bool,
    pub child_selected: usize,
    pub child_state: ListState,
}

impl Default for Menu {
    fn default() -> Self {
        let items = vec![
            MenuItem {
                label: "📱 List Devices".to_string(),
                description: "Show all connected Android devices with their status".to_string(),
                command: AdbCommand::ListDevices,
                children: vec![
                    MenuChild {
                        label: "📋 List Connected Devices".to_string(),
                        description: "Show basic device list".to_string(),
                        command: AdbCommand::ListDevices,
                    },
                    MenuChild {
                        label: "📝 Detailed Device Info".to_string(),
                        description: "Show devices with detailed information".to_string(),
                        command: AdbCommand::GetDeviceState,
                    },
                    MenuChild {
                        label: "🔍 Device Serial Numbers".to_string(),
                        description: "List only device serial numbers".to_string(),
                        command: AdbCommand::GetSerialNumber,
                    },
                ],
            },
            MenuItem {
                label: "📋 List Packages".to_string(),
                description: "List all installed packages on the device".to_string(),
                command: AdbCommand::ListPackages {
                    include_path: true,
                    filter: PackageFilter::All,
                },
                children: vec![
                    MenuChild {
                        label: "📦 All Packages".to_string(),
                        description: "List all installed packages".to_string(),
                        command: AdbCommand::ListPackages {
                            include_path: false,
                            filter: PackageFilter::All,
                        },
                    },
                    MenuChild {
                        label: "📁 Packages with Paths".to_string(),
                        description: "List packages with file paths".to_string(),
                        command: AdbCommand::ListPackages {
                            include_path: true,
                            filter: PackageFilter::All,
                        },
                    },
                    MenuChild {
                        label: "👤 User Packages Only".to_string(),
                        description: "List only user-installed packages".to_string(),
                        command: AdbCommand::ListPackages {
                            include_path: false,
                            filter: PackageFilter::User,
                        },
                    },
                    MenuChild {
                        label: "⚙️ System Packages Only".to_string(),
                        description: "List only system packages".to_string(),
                        command: AdbCommand::ListPackages {
                            include_path: false,
                            filter: PackageFilter::System,
                        },
                    },
                ],
            },
            MenuItem {
                label: "🔋 Battery Info".to_string(),
                description: "Display detailed battery information".to_string(),
                command: AdbCommand::GetBatteryInfo,
                children: vec![
                    MenuChild {
                        label: "🔋 Full Battery Status".to_string(),
                        description: "Complete battery information".to_string(),
                        command: AdbCommand::GetBatteryInfo,
                    },
                    MenuChild {
                        label: "⚡ Battery Level".to_string(),
                        description: "Show just battery percentage".to_string(),
                        command: AdbCommand::Shell {
                            command: "dumpsys battery | grep level".to_string(),
                        },
                    },
                    MenuChild {
                        label: "🔌 Charging Status".to_string(),
                        description: "Show charging state".to_string(),
                        command: AdbCommand::Shell {
                            command: "dumpsys battery | grep 'AC powered\\|USB powered\\|status'"
                                .to_string(),
                        },
                    },
                ],
            },
            MenuItem {
                label: "💾 Memory Usage".to_string(),
                description: "Show memory usage statistics".to_string(),
                command: AdbCommand::GetMemoryInfo,
                children: vec![
                    MenuChild {
                        label: "📊 System Memory".to_string(),
                        description: "Overall system memory usage".to_string(),
                        command: AdbCommand::GetMemoryInfo,
                    },
                    MenuChild {
                        label: "📱 Available Memory".to_string(),
                        description: "Show available memory".to_string(),
                        command: AdbCommand::Shell {
                            command: "cat /proc/meminfo | grep -E 'MemTotal|MemFree|MemAvailable'"
                                .to_string(),
                        },
                    },
                    MenuChild {
                        label: "🔝 Top Memory Apps".to_string(),
                        description: "Apps using most memory".to_string(),
                        command: AdbCommand::Shell {
                            command: "dumpsys meminfo | grep -E '(Total|TOTAL)' | head -10"
                                .to_string(),
                        },
                    },
                ],
            },
            MenuItem {
                label: "📊 CPU Info".to_string(),
                description: "Display CPU information and usage".to_string(),
                command: AdbCommand::GetCpuInfo,
                children: vec![
                    MenuChild {
                        label: "🔧 CPU Details".to_string(),
                        description: "Detailed CPU information".to_string(),
                        command: AdbCommand::GetCpuInfo,
                    },
                    MenuChild {
                        label: "⚡ CPU Usage".to_string(),
                        description: "Top processes by CPU usage".to_string(),
                        command: AdbCommand::Shell {
                            command: "ps -eo PID,PPID,USER,COMM | head -20".to_string(),
                        },
                    },
                    MenuChild {
                        label: "📈 Load Average".to_string(),
                        description: "System load average".to_string(),
                        command: AdbCommand::Shell {
                            command: "cat /proc/loadavg".to_string(),
                        },
                    },
                ],
            },
            MenuItem {
                label: "🔗 Network Info".to_string(),
                description: "Show network connectivity information".to_string(),
                command: AdbCommand::GetNetworkInfo,
                children: vec![
                    MenuChild {
                        label: "🌐 Connectivity Status".to_string(),
                        description: "Network connectivity details".to_string(),
                        command: AdbCommand::GetNetworkInfo,
                    },
                    MenuChild {
                        label: "📶 WiFi Info".to_string(),
                        description: "WiFi connection details".to_string(),
                        command: AdbCommand::GetWifiStatus,
                    },
                    MenuChild {
                        label: "🔗 IP Configuration".to_string(),
                        description: "Network interface configuration".to_string(),
                        command: AdbCommand::Shell {
                            command: "ip addr show".to_string(),
                        },
                    },
                ],
            },
            MenuItem {
                label: "📱 Device Properties".to_string(),
                description: "Get all device system properties".to_string(),
                command: AdbCommand::GetDeviceProperties,
                children: vec![
                    MenuChild {
                        label: "📋 All Properties".to_string(),
                        description: "Show all system properties".to_string(),
                        command: AdbCommand::GetDeviceProperties,
                    },
                    MenuChild {
                        label: "🏷️ Device Model".to_string(),
                        description: "Show device model and brand".to_string(),
                        command: AdbCommand::Shell {
                            command: "getprop | grep -E 'ro.product.model|ro.product.brand|ro.product.name'".to_string(),
                        },
                    },
                    MenuChild {
                        label: "🔢 Android Version".to_string(),
                        description: "Show Android version info".to_string(),
                        command: AdbCommand::Shell {
                            command: "getprop | grep -E 'ro.build.version|ro.build.id'"
                                .to_string(),
                        },
                    },
                ],
            },
            MenuItem {
                label: "🎯 Running Processes".to_string(),
                description: "List all running processes".to_string(),
                command: AdbCommand::ListProcesses,
                children: vec![
                    MenuChild {
                        label: "📋 All Processes".to_string(),
                        description: "List all running processes".to_string(),
                        command: AdbCommand::ListProcesses,
                    },
                    MenuChild {
                        label: "🔝 Top Processes".to_string(),
                        description: "Top 20 running processes".to_string(),
                        command: AdbCommand::Shell {
                            command: "ps -A | head -20".to_string(),
                        },
                    },
                    MenuChild {
                        label: "👤 User Processes".to_string(),
                        description: "User application processes only".to_string(),
                        command: AdbCommand::Shell {
                            command: "ps -A | grep -v 'system' | grep -v 'root' | head -15"
                                .to_string(),
                        },
                    },
                    MenuChild {
                        label: "📊 Process Details".to_string(),
                        description: "Detailed process information with formatting".to_string(),
                        command: AdbCommand::Shell {
                            command: "echo 'PID    USER         COMMAND'; echo '---    ----         -------'; ps -A | awk '{printf \"%-6s %-12s %s\\n\", $2, $1, $9}' | head -20".to_string(),
                        },
                    },
                ],
            },
            MenuItem {
                label: "📊 System Services".to_string(),
                description: "List all system services status".to_string(),
                command: AdbCommand::Shell {
                    command: "service list".to_string(),
                },
                children: vec![
                    MenuChild {
                        label: "📋 All Services".to_string(),
                        description: "List all system services".to_string(),
                        command: AdbCommand::Shell {
                            command: "service list".to_string(),
                        },
                    },
                    MenuChild {
                        label: "🔧 Running Services".to_string(),
                        description: "Show only running services".to_string(),
                        command: AdbCommand::Shell {
                            command: "dumpsys activity services".to_string(),
                        },
                    },
                    MenuChild {
                        label: "📱 App Services".to_string(),
                        description: "Application services only".to_string(),
                        command: AdbCommand::Shell {
                            command: "dumpsys activity services | grep -A 5 'ServiceRecord'"
                                .to_string(),
                        },
                    },
                ],
            },
            MenuItem {
                label: "📷 Screenshot".to_string(),
                description: "Take and save device screenshots".to_string(),
                command: AdbCommand::TakeScreenshot,
                children: vec![
                    MenuChild {
                        label: "📸 Take Screenshot".to_string(),
                        description: "Take screenshot and save to device".to_string(),
                        command: AdbCommand::TakeScreenshot,
                    },
                    MenuChild {
                        label: "📐 Screen Resolution".to_string(),
                        description: "Show screen size and density".to_string(),
                        command: AdbCommand::GetScreenResolution,
                    },
                    MenuChild {
                        label: "🖼️ View Screenshot Path".to_string(),
                        description: "Show where screenshots are saved".to_string(),
                        command: AdbCommand::Shell {
                            command: "ls -la /sdcard/screenshot*.png".to_string(),
                        },
                    },
                ],
            },
            MenuItem {
                label: "🔄 Reboot Device".to_string(),
                description: "Reboot the connected device".to_string(),
                command: AdbCommand::Shell {
                    command: "reboot".to_string(),
                },
                children: vec![
                    MenuChild {
                        label: "🔄 Normal Reboot".to_string(),
                        description: "Reboot device normally".to_string(),
                        command: AdbCommand::Shell {
                            command: "reboot".to_string(),
                        },
                    },
                    MenuChild {
                        label: "⚡ Fast Reboot".to_string(),
                        description: "Fast reboot (bootloader)".to_string(),
                        command: AdbCommand::Shell {
                            command: "reboot bootloader".to_string(),
                        },
                    },
                    MenuChild {
                        label: "🔧 Recovery Mode".to_string(),
                        description: "Reboot to recovery mode".to_string(),
                        command: AdbCommand::Shell {
                            command: "reboot recovery".to_string(),
                        },
                    },
                ],
            },
            MenuItem {
                label: "📜 System Log".to_string(),
                description: "View recent system logs (last 100 lines)".to_string(),
                command: AdbCommand::GetSystemLog { lines: 100 },
                children: vec![
                    MenuChild {
                        label: "📜 Recent Logs".to_string(),
                        description: "Last 100 log entries".to_string(),
                        command: AdbCommand::GetSystemLog { lines: 100 },
                    },
                    MenuChild {
                        label: "🚨 Error Logs Only".to_string(),
                        description: "Show only error messages".to_string(),
                        command: AdbCommand::Shell {
                            command: "logcat -d *:E".to_string(),
                        },
                    },
                    MenuChild {
                        label: "⚠️ Warning and Error".to_string(),
                        description: "Show warnings and errors".to_string(),
                        command: AdbCommand::Shell {
                            command: "logcat -d *:W".to_string(),
                        },
                    },
                    MenuChild {
                        label: "🔄 Clear Logs".to_string(),
                        description: "Clear the log buffer".to_string(),
                        command: AdbCommand::Shell {
                            command: "logcat -c".to_string(),
                        },
                    },
                ],
            },
            MenuItem {
                label: "🔍 ADB Version".to_string(),
                description: "Display ADB version information".to_string(),
                command: AdbCommand::GetAdbVersion,
                children: vec![
                    MenuChild {
                        label: "🔍 ADB Version".to_string(),
                        description: "Show ADB version".to_string(),
                        command: AdbCommand::GetAdbVersion,
                    },
                    MenuChild {
                        label: "📋 Server Status".to_string(),
                        description: "Check ADB server status".to_string(),
                        command: AdbCommand::ListDevices,
                    },
                ],
            },
        ];

        let mut state = ListState::default();
        state.select(Some(0));

        let mut child_state = ListState::default();
        child_state.select(Some(0));

        Self {
            items,
            state,
            selected: 0,
            tick_count: 0,
            last_selected: 0,
            position_change_boost: 0,
            in_child_mode: false,
            child_selected: 0,
            child_state,
        }
    }
}

impl Menu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    pub fn next(&mut self) {
        let prev = self.selected;

        if self.in_child_mode {
            let child_count = self.items[self.selected].children.len();
            if self.child_selected < child_count - 1 {
                self.child_selected += 1;
                self.child_state.select(Some(self.child_selected));
            }
        } else if self.selected < self.items.len() - 1 {
            self.selected += 1;
            self.state.select(Some(self.selected));
        }

        // Track movement for effects
        if prev != self.selected {
            self.position_change_boost = 30;
            self.last_selected = prev;
        }
    }

    pub fn previous(&mut self) {
        let prev = self.selected;

        if self.in_child_mode {
            if self.child_selected > 0 {
                self.child_selected -= 1;
                self.child_state.select(Some(self.child_selected));
            }
        } else if self.selected > 0 {
            self.selected -= 1;
            self.state.select(Some(self.selected));
        }

        // Track movement for effects
        if prev != self.selected {
            self.position_change_boost = 30;
            self.last_selected = prev;
        }
    }

    pub fn get_selected_item(&self) -> &MenuItem {
        &self.items[self.selected]
    }

    pub fn get_selected_command(&self) -> AdbCommand {
        if self.in_child_mode && !self.items[self.selected].children.is_empty() {
            self.items[self.selected].children[self.child_selected]
                .command
                .clone()
        } else {
            self.items[self.selected].command.clone()
        }
    }

    pub fn enter_child_mode(&mut self) {
        if !self.items[self.selected].children.is_empty() {
            self.in_child_mode = true;
            self.child_selected = 0;
            self.child_state.select(Some(0));
        }
    }

    pub fn trigger_fade_in(&mut self) {
        self.position_change_boost = 60;
    }

    pub fn exit_child_mode(&mut self) {
        if self.in_child_mode {
            self.in_child_mode = false;
            self.child_selected = 0;
        }
    }

    pub fn is_in_child_mode(&self) -> bool {
        self.in_child_mode
    }
}

impl Widget for &Menu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create border block
        let block = Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(61, 220, 132)))
            .title(" ADB Commands ");

        let inner = block.inner(area);
        block.render(area, buf);

        if self.in_child_mode {
            // Render child menu
            let selected_item = &self.items[self.selected];

            // Split area for parent indicator and child menu
            let chunks = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(inner);

            // Render parent item indicator
            let parent_text = format!("← {} (Press Backspace to return)", selected_item.label);
            let parent_para = Paragraph::new(parent_text)
                .style(
                    Style::default()
                        .fg(Color::Rgb(61, 220, 132))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Left);
            parent_para.render(chunks[0], buf);

            // Render child items
            let child_items: Vec<ListItem> = selected_item
                .children
                .iter()
                .enumerate()
                .map(|(idx, child)| {
                    let is_selected = idx == self.child_selected;
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Rgb(61, 220, 132))
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let content =
                        Line::from(vec![Span::raw("  "), Span::styled(&child.label, style)]);

                    ListItem::new(content)
                })
                .collect();

            let child_list = List::new(child_items);
            child_list.render(chunks[1], buf);

            // Render description at bottom
            if !selected_item.children.is_empty() {
                let selected_child = &selected_item.children[self.child_selected];
                let desc_area = Rect {
                    x: inner.x,
                    y: inner.y + inner.height.saturating_sub(3),
                    width: inner.width,
                    height: 3,
                };

                let desc_text = format!("  {}", selected_child.description);
                let desc_para = Paragraph::new(desc_text)
                    .style(Style::default().fg(Color::Gray))
                    .alignment(Alignment::Left);
                desc_para.render(desc_area, buf);
            }
        } else {
            // Render main menu
            let items: Vec<ListItem> = self
                .items
                .iter()
                .enumerate()
                .map(|(idx, item)| {
                    let is_selected = idx == self.selected;
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Rgb(61, 220, 132))
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let content = Line::from(vec![
                        Span::raw("  "),
                        Span::styled(&item.label, style),
                        if !item.children.is_empty() {
                            Span::raw(" ▶")
                        } else {
                            Span::raw("")
                        },
                    ]);

                    ListItem::new(content)
                })
                .collect();

            let list = List::new(items);
            list.render(inner, buf);

            // Render description at bottom
            let selected_item = &self.items[self.selected];
            let desc_area = Rect {
                x: inner.x,
                y: inner.y + inner.height.saturating_sub(3),
                width: inner.width,
                height: 3,
            };

            let desc_text = format!("  {}", selected_item.description);
            let desc_para = Paragraph::new(desc_text)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Left);
            desc_para.render(desc_area, buf);
        }
    }
}
