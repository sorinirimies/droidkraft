//! Main Menu Example
//!
//! This example demonstrates how to create and use the droidtui main menu system.
//! It shows the menu structure, navigation, and command execution capabilities.
//!
//! Run with: cargo run --example main_menu

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use droidtui::adb::{AdbCommand, PackageFilter};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::{self, stdout};

/// Android green color used throughout droidtui
const ANDROID_GREEN: Color = Color::Rgb(61, 220, 132);

/// Main application state
struct App {
    selected: usize,
    menu_items: Vec<MenuItem>,
    should_quit: bool,
    show_help: bool,
}

/// Represents a menu item with label and description
struct MenuItem {
    label: String,
    description: String,
    command: AdbCommand,
}

impl App {
    fn new() -> Self {
        Self {
            selected: 0,
            menu_items: vec![
                MenuItem {
                    label: "📱 List Devices".to_string(),
                    description: "Show all connected Android devices with their status and details"
                        .to_string(),
                    command: AdbCommand::ListDevices,
                },
                MenuItem {
                    label: "📋 List Packages".to_string(),
                    description: "List all installed packages on the device with file paths"
                        .to_string(),
                    command: AdbCommand::ListPackages {
                        include_path: true,
                        filter: PackageFilter::All,
                    },
                },
                MenuItem {
                    label: "🔋 Battery Info".to_string(),
                    description: "Display detailed battery information including level, temperature, and charging status"
                        .to_string(),
                    command: AdbCommand::GetBatteryInfo,
                },
                MenuItem {
                    label: "💾 Memory Usage".to_string(),
                    description: "Show comprehensive memory usage statistics for system and apps"
                        .to_string(),
                    command: AdbCommand::GetMemoryInfo,
                },
                MenuItem {
                    label: "📊 CPU Info".to_string(),
                    description: "Display CPU information, specifications, and current usage"
                        .to_string(),
                    command: AdbCommand::GetCpuInfo,
                },
                MenuItem {
                    label: "🔗 Network Info".to_string(),
                    description: "Show network connectivity status and configuration details"
                        .to_string(),
                    command: AdbCommand::GetNetworkInfo,
                },
                MenuItem {
                    label: "📱 Device Properties".to_string(),
                    description: "Get all device system properties including model and Android version"
                        .to_string(),
                    command: AdbCommand::GetDeviceProperties,
                },
                MenuItem {
                    label: "🎯 Running Processes".to_string(),
                    description: "List all currently running processes on the device"
                        .to_string(),
                    command: AdbCommand::ListProcesses,
                },
                MenuItem {
                    label: "📊 System Services".to_string(),
                    description: "List all system services and their current status"
                        .to_string(),
                    command: AdbCommand::Shell {
                        command: "service list".to_string(),
                    },
                },
                MenuItem {
                    label: "📷 Screenshot".to_string(),
                    description: "Take a screenshot and save it locally"
                        .to_string(),
                    command: AdbCommand::TakeScreenshot,
                },
                MenuItem {
                    label: "🔄 Reboot Device".to_string(),
                    description: "Reboot the connected Android device"
                        .to_string(),
                    command: AdbCommand::Shell {
                        command: "reboot".to_string(),
                    },
                },
                MenuItem {
                    label: "📜 System Log".to_string(),
                    description: "View recent system logs (last 100 lines)"
                        .to_string(),
                    command: AdbCommand::GetSystemLog { lines: 100 },
                },
                MenuItem {
                    label: "🔍 ADB Version".to_string(),
                    description: "Display ADB version information"
                        .to_string(),
                    command: AdbCommand::GetAdbVersion,
                },
            ],
            should_quit: false,
            show_help: false,
        }
    }

    fn next(&mut self) {
        if self.selected < self.menu_items.len() - 1 {
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

    fn get_selected_item(&self) -> &MenuItem {
        &self.menu_items[self.selected]
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
                    KeyCode::Enter => {
                        // In a real implementation, this would execute the command
                        // For this example, we just show which command would be executed
                        let item = app.get_selected_item();
                        eprintln!("Would execute: {:?}", item.command);
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

    // Render content (menu or help)
    if app.show_help {
        render_help(frame, chunks[1]);
    } else {
        render_menu(frame, chunks[1], app);
    }

    // Render footer
    render_footer(frame, chunks[2], app.show_help);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled("🤖 ", Style::default().fg(ANDROID_GREEN)),
        Span::styled(
            "DroidTUI ",
            Style::default()
                .fg(ANDROID_GREEN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("- Main Menu Example", Style::default().fg(Color::White)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ANDROID_GREEN)),
    )
    .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(title, area);
}

fn render_menu(frame: &mut Frame, area: Rect, app: &App) {
    // Split into left (menu) and right (description) panels
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Render menu list
    let menu_items: Vec<ListItem> = app
        .menu_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
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
                Span::styled(&item.label, style),
                Span::raw(" ▶"),
            ]))
        })
        .collect();

    let menu_list = List::new(menu_items).block(
        Block::default()
            .title(" ADB Commands ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ANDROID_GREEN)),
    );

    frame.render_widget(menu_list, chunks[0]);

    // Render description panel
    let selected_item = &app.menu_items[app.selected];
    let description = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "Command Type: ",
            Style::default()
                .fg(ANDROID_GREEN)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            format!("{:?}", selected_item.command),
            Style::default().fg(Color::Yellow),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Description:",
            Style::default()
                .fg(ANDROID_GREEN)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            &selected_item.description,
            Style::default().fg(Color::White),
        )]),
    ])
    .block(
        Block::default()
            .title(" Details ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    )
    .wrap(Wrap { trim: true });

    frame.render_widget(description, chunks[1]);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
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
            Span::raw("      - Move up"),
        ]),
        Line::from(vec![
            Span::styled("  ↓/j", Style::default().fg(ANDROID_GREEN)),
            Span::raw("      - Move down"),
        ]),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(ANDROID_GREEN)),
            Span::raw("    - Execute selected command"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Actions:",
            Style::default()
                .fg(ANDROID_GREEN)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  h/?", Style::default().fg(ANDROID_GREEN)),
            Span::raw("      - Toggle this help screen"),
        ]),
        Line::from(vec![
            Span::styled("  q/Esc", Style::default().fg(ANDROID_GREEN)),
            Span::raw("    - Quit application"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "About:",
            Style::default()
                .fg(ANDROID_GREEN)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("This example demonstrates the droidtui menu system."),
        Line::from("In the full application, commands are executed via ADB"),
        Line::from("and results are displayed in a scrollable panel."),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ANDROID_GREEN)),
        )
        .alignment(ratatui::layout::Alignment::Left);

    frame.render_widget(help, area);
}

fn render_footer(frame: &mut Frame, area: Rect, show_help: bool) {
    let footer_text = if show_help {
        "Press h/? to return to menu | Press q/Esc to quit"
    } else {
        "↑/↓: Navigate | Enter: Execute | h/?: Help | q/Esc: Quit"
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
    .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(footer, area);
}
