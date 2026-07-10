//! Device Information Example
//!
//! This example demonstrates how to query and display various device information
//! from connected Android devices using ADB commands.
//!
//! Run with: cargo run --example device_info

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::{self, stdout};

/// Android green color
const ANDROID_GREEN: Color = Color::Rgb(61, 220, 132);

/// Device information category
struct InfoCategory {
    label: String,
    icon: String,
    description: String,
    command: String,
    info_type: String,
}

/// Application state
struct App {
    selected: usize,
    categories: Vec<InfoCategory>,
    should_quit: bool,
    show_help: bool,
}

impl App {
    fn new() -> Self {
        Self {
            selected: 0,
            categories: vec![
                InfoCategory {
                    label: "Device Model".to_string(),
                    icon: "📱".to_string(),
                    description: "Display device manufacturer, model name, and product information"
                        .to_string(),
                    command: "adb shell getprop ro.product.model".to_string(),
                    info_type: "Hardware".to_string(),
                },
                InfoCategory {
                    label: "Android Version".to_string(),
                    icon: "🤖".to_string(),
                    description: "Show Android OS version and SDK level running on the device"
                        .to_string(),
                    command: "adb shell getprop ro.build.version.release".to_string(),
                    info_type: "System".to_string(),
                },
                InfoCategory {
                    label: "Build Number".to_string(),
                    icon: "🏗️".to_string(),
                    description: "Display the build number and fingerprint of the current ROM"
                        .to_string(),
                    command: "adb shell getprop ro.build.display.id".to_string(),
                    info_type: "System".to_string(),
                },
                InfoCategory {
                    label: "Serial Number".to_string(),
                    icon: "🔢".to_string(),
                    description: "Get the unique serial number identifier for this device"
                        .to_string(),
                    command: "adb get-serialno".to_string(),
                    info_type: "Device ID".to_string(),
                },
                InfoCategory {
                    label: "Screen Resolution".to_string(),
                    icon: "📐".to_string(),
                    description: "Display screen resolution and density (DPI) information"
                        .to_string(),
                    command: "adb shell wm size && adb shell wm density".to_string(),
                    info_type: "Display".to_string(),
                },
                InfoCategory {
                    label: "CPU Architecture".to_string(),
                    icon: "⚙️".to_string(),
                    description: "Show CPU architecture (ARM, ARM64, x86, etc.) and capabilities"
                        .to_string(),
                    command: "adb shell getprop ro.product.cpu.abi".to_string(),
                    info_type: "Hardware".to_string(),
                },
                InfoCategory {
                    label: "Memory Info".to_string(),
                    icon: "💾".to_string(),
                    description: "Display total RAM and available memory on the device".to_string(),
                    command: "adb shell cat /proc/meminfo | grep MemTotal".to_string(),
                    info_type: "Hardware".to_string(),
                },
                InfoCategory {
                    label: "Storage Space".to_string(),
                    icon: "💿".to_string(),
                    description: "Show internal storage capacity and available free space"
                        .to_string(),
                    command: "adb shell df /data".to_string(),
                    info_type: "Storage".to_string(),
                },
                InfoCategory {
                    label: "Battery Status".to_string(),
                    icon: "🔋".to_string(),
                    description: "Current battery level, temperature, and charging status"
                        .to_string(),
                    command: "adb shell dumpsys battery | grep level".to_string(),
                    info_type: "Power".to_string(),
                },
                InfoCategory {
                    label: "WiFi Status".to_string(),
                    icon: "📶".to_string(),
                    description: "Display WiFi connection status and IP address information"
                        .to_string(),
                    command: "adb shell ip addr show wlan0".to_string(),
                    info_type: "Network".to_string(),
                },
                InfoCategory {
                    label: "Bluetooth Info".to_string(),
                    icon: "🔵".to_string(),
                    description: "Show Bluetooth status and paired devices information".to_string(),
                    command: "adb shell dumpsys bluetooth_manager".to_string(),
                    info_type: "Network".to_string(),
                },
                InfoCategory {
                    label: "IMEI Number".to_string(),
                    icon: "📞".to_string(),
                    description: "Display IMEI number for cellular-enabled devices".to_string(),
                    command: "adb shell service call iphonesubinfo 1".to_string(),
                    info_type: "Device ID".to_string(),
                },
                InfoCategory {
                    label: "Uptime".to_string(),
                    icon: "⏱️".to_string(),
                    description: "Show how long the device has been running since last reboot"
                        .to_string(),
                    command: "adb shell uptime".to_string(),
                    info_type: "System".to_string(),
                },
                InfoCategory {
                    label: "Kernel Version".to_string(),
                    icon: "🐧".to_string(),
                    description: "Display Linux kernel version running on the device".to_string(),
                    command: "adb shell uname -a".to_string(),
                    info_type: "System".to_string(),
                },
                InfoCategory {
                    label: "Language & Locale".to_string(),
                    icon: "🌍".to_string(),
                    description: "Show current language and locale settings".to_string(),
                    command: "adb shell getprop persist.sys.locale".to_string(),
                    info_type: "System".to_string(),
                },
                InfoCategory {
                    label: "Security Patch".to_string(),
                    icon: "🔒".to_string(),
                    description: "Display the date of the latest Android security patch"
                        .to_string(),
                    command: "adb shell getprop ro.build.version.security_patch".to_string(),
                    info_type: "Security".to_string(),
                },
            ],
            should_quit: false,
            show_help: false,
        }
    }

    fn next(&mut self) {
        if self.selected < self.categories.len() - 1 {
            self.selected += 1;
        }
    }

    fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    fn get_selected_category(&self) -> &InfoCategory {
        &self.categories[self.selected]
    }
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(ratatui::backend::CrosstermBackend::new(stdout()))?;

    let mut app = App::new();

    // Main loop
    loop {
        terminal.draw(|frame| ui(frame, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        app.should_quit = true;
                    }
                    KeyCode::Char('h') | KeyCode::Char('?') => {
                        app.toggle_help();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.next();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.previous();
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        let category = app.get_selected_category();
                        eprintln!("Would execute: {}", category.command);
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

fn ui(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Render title
    render_title(frame, chunks[0]);

    // Render content
    if app.show_help {
        render_help(frame, chunks[1]);
    } else {
        render_categories(frame, chunks[1], app);
    }

    // Render footer
    render_footer(frame, chunks[2], app.show_help);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled("📱 ", Style::default().fg(ANDROID_GREEN)),
        Span::styled(
            "Device Information ",
            Style::default()
                .fg(ANDROID_GREEN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("- DroidTUI Example", Style::default().fg(Color::White)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ANDROID_GREEN)),
    )
    .alignment(Alignment::Center);

    frame.render_widget(title, area);
}

fn render_categories(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Render category list
    let items: Vec<ListItem> = app
        .categories
        .iter()
        .enumerate()
        .map(|(i, category)| {
            let style = if i == app.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(ANDROID_GREEN)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(&category.icon, Style::default()),
                Span::raw(" "),
                Span::styled(&category.label, style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Information Categories ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ANDROID_GREEN)),
    );

    frame.render_widget(list, chunks[0]);

    // Render details panel
    let selected = &app.categories[app.selected];

    let type_color = match selected.info_type.as_str() {
        "Hardware" => Color::Cyan,
        "System" => Color::Yellow,
        "Network" => Color::Blue,
        "Storage" => Color::Magenta,
        "Power" => Color::Green,
        "Security" => Color::Red,
        "Device ID" => Color::LightYellow,
        "Display" => Color::LightBlue,
        _ => Color::White,
    };

    let details = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(&selected.icon, Style::default().fg(ANDROID_GREEN)),
            Span::raw(" "),
            Span::styled(
                &selected.label,
                Style::default()
                    .fg(ANDROID_GREEN)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Category: ",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&selected.info_type, Style::default().fg(type_color)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Description:",
            Style::default()
                .fg(ANDROID_GREEN)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            &selected.description,
            Style::default().fg(Color::White),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Command:",
            Style::default()
                .fg(ANDROID_GREEN)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            &selected.command,
            Style::default().fg(Color::Yellow),
        )]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "💡 Tip:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "Press Enter to query this information",
            Style::default().fg(Color::Gray),
        )]),
    ])
    .block(
        Block::default()
            .title(" Details ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    )
    .wrap(Wrap { trim: true });

    frame.render_widget(details, chunks[1]);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Device Information Query",
            Style::default()
                .fg(ANDROID_GREEN)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("This example shows how to query various device information using ADB."),
        Line::from("Each category represents a different aspect of your Android device."),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Information Categories:",
            Style::default()
                .fg(ANDROID_GREEN)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Hardware", Style::default().fg(Color::Cyan)),
            Span::raw("  - Physical device specifications"),
        ]),
        Line::from(vec![
            Span::styled("  System", Style::default().fg(Color::Yellow)),
            Span::raw("    - OS and software information"),
        ]),
        Line::from(vec![
            Span::styled("  Network", Style::default().fg(Color::Blue)),
            Span::raw("    - Connectivity status"),
        ]),
        Line::from(vec![
            Span::styled("  Storage", Style::default().fg(Color::Magenta)),
            Span::raw("    - Disk space information"),
        ]),
        Line::from(vec![
            Span::styled("  Power", Style::default().fg(Color::Green)),
            Span::raw("      - Battery and charging"),
        ]),
        Line::from(vec![
            Span::styled("  Security", Style::default().fg(Color::Red)),
            Span::raw("   - Security patch level"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default()
                .fg(ANDROID_GREEN)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↑/k", Style::default().fg(ANDROID_GREEN)),
            Span::raw("          - Move up"),
        ]),
        Line::from(vec![
            Span::styled("  ↓/j", Style::default().fg(ANDROID_GREEN)),
            Span::raw("          - Move down"),
        ]),
        Line::from(vec![
            Span::styled("  Enter/Space", Style::default().fg(ANDROID_GREEN)),
            Span::raw("   - Query selected information"),
        ]),
        Line::from(vec![
            Span::styled("  h/?", Style::default().fg(ANDROID_GREEN)),
            Span::raw("          - Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("  q/Esc", Style::default().fg(ANDROID_GREEN)),
            Span::raw("        - Quit application"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ANDROID_GREEN)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(help, area);
}

fn render_footer(frame: &mut Frame, area: Rect, show_help: bool) {
    let footer_text = if show_help {
        "Press h/? to return | q/Esc to quit"
    } else {
        "↑/↓: Navigate | Enter/Space: Query Info | h/?: Help | q/Esc: Quit"
    };

    let footer = Paragraph::new(Line::from(vec![Span::styled(
        footer_text,
        Style::default().fg(Color::White),
    )]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    )
    .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}
