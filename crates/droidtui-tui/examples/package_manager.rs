//! Package Manager Example
//!
//! This example demonstrates package management capabilities for Android devices.
//! It shows how to list, search, and manage installed packages using ADB.
//!
//! Run with: cargo run --example package_manager

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

/// Package management action
struct PackageAction {
    label: String,
    icon: String,
    description: String,
    command: String,
    category: String,
    requires_confirm: bool,
}

/// Application state
struct App {
    selected: usize,
    actions: Vec<PackageAction>,
    should_quit: bool,
    show_help: bool,
}

impl App {
    fn new() -> Self {
        Self {
            selected: 0,
            actions: vec![
                PackageAction {
                    label: "List All Packages".to_string(),
                    icon: "📦".to_string(),
                    description: "Display all installed packages on the device".to_string(),
                    command: "adb shell pm list packages".to_string(),
                    category: "Query".to_string(),
                    requires_confirm: false,
                },
                PackageAction {
                    label: "List User Apps".to_string(),
                    icon: "👤".to_string(),
                    description: "Show only user-installed applications (3rd party apps)"
                        .to_string(),
                    command: "adb shell pm list packages -3".to_string(),
                    category: "Query".to_string(),
                    requires_confirm: false,
                },
                PackageAction {
                    label: "List System Apps".to_string(),
                    icon: "⚙️".to_string(),
                    description: "Show only system pre-installed applications".to_string(),
                    command: "adb shell pm list packages -s".to_string(),
                    category: "Query".to_string(),
                    requires_confirm: false,
                },
                PackageAction {
                    label: "List Enabled Apps".to_string(),
                    icon: "✅".to_string(),
                    description: "Display only enabled packages".to_string(),
                    command: "adb shell pm list packages -e".to_string(),
                    category: "Query".to_string(),
                    requires_confirm: false,
                },
                PackageAction {
                    label: "List Disabled Apps".to_string(),
                    icon: "🚫".to_string(),
                    description: "Display only disabled packages".to_string(),
                    command: "adb shell pm list packages -d".to_string(),
                    category: "Query".to_string(),
                    requires_confirm: false,
                },
                PackageAction {
                    label: "List with Paths".to_string(),
                    icon: "📁".to_string(),
                    description: "Show packages with their file system paths".to_string(),
                    command: "adb shell pm list packages -f".to_string(),
                    category: "Query".to_string(),
                    requires_confirm: false,
                },
                PackageAction {
                    label: "List with Installers".to_string(),
                    icon: "📲".to_string(),
                    description: "Show packages with installer information".to_string(),
                    command: "adb shell pm list packages -i".to_string(),
                    category: "Query".to_string(),
                    requires_confirm: false,
                },
                PackageAction {
                    label: "Package Info".to_string(),
                    icon: "ℹ️".to_string(),
                    description: "Get detailed information about a specific package".to_string(),
                    command: "adb shell dumpsys package <package_name>".to_string(),
                    category: "Info".to_string(),
                    requires_confirm: false,
                },
                PackageAction {
                    label: "Package Permissions".to_string(),
                    icon: "🔐".to_string(),
                    description: "List all permissions used by a package".to_string(),
                    command: "adb shell dumpsys package <package_name> | grep permission"
                        .to_string(),
                    category: "Info".to_string(),
                    requires_confirm: false,
                },
                PackageAction {
                    label: "Clear App Data".to_string(),
                    icon: "🧹".to_string(),
                    description: "Clear all data for a specific application".to_string(),
                    command: "adb shell pm clear <package_name>".to_string(),
                    category: "Manage".to_string(),
                    requires_confirm: true,
                },
                PackageAction {
                    label: "Clear App Cache".to_string(),
                    icon: "💨".to_string(),
                    description: "Clear only the cache for an application".to_string(),
                    command: "adb shell pm trim-caches <package_name>".to_string(),
                    category: "Manage".to_string(),
                    requires_confirm: false,
                },
                PackageAction {
                    label: "Disable App".to_string(),
                    icon: "⏸️".to_string(),
                    description: "Disable an application without uninstalling it".to_string(),
                    command: "adb shell pm disable-user <package_name>".to_string(),
                    category: "Manage".to_string(),
                    requires_confirm: true,
                },
                PackageAction {
                    label: "Enable App".to_string(),
                    icon: "▶️".to_string(),
                    description: "Re-enable a previously disabled application".to_string(),
                    command: "adb shell pm enable <package_name>".to_string(),
                    category: "Manage".to_string(),
                    requires_confirm: false,
                },
                PackageAction {
                    label: "Uninstall App".to_string(),
                    icon: "🗑️".to_string(),
                    description: "Uninstall an application from the device".to_string(),
                    command: "adb uninstall <package_name>".to_string(),
                    category: "Manage".to_string(),
                    requires_confirm: true,
                },
                PackageAction {
                    label: "Install APK".to_string(),
                    icon: "📥".to_string(),
                    description: "Install an APK file to the device".to_string(),
                    command: "adb install <path_to_apk>".to_string(),
                    category: "Manage".to_string(),
                    requires_confirm: false,
                },
                PackageAction {
                    label: "Grant Permission".to_string(),
                    icon: "🔓".to_string(),
                    description: "Grant a specific permission to an app".to_string(),
                    command: "adb shell pm grant <package_name> <permission>".to_string(),
                    category: "Permissions".to_string(),
                    requires_confirm: false,
                },
                PackageAction {
                    label: "Revoke Permission".to_string(),
                    icon: "🔒".to_string(),
                    description: "Revoke a specific permission from an app".to_string(),
                    command: "adb shell pm revoke <package_name> <permission>".to_string(),
                    category: "Permissions".to_string(),
                    requires_confirm: true,
                },
                PackageAction {
                    label: "List Permissions".to_string(),
                    icon: "📋".to_string(),
                    description: "List all available permissions on the device".to_string(),
                    command: "adb shell pm list permissions -g".to_string(),
                    category: "Permissions".to_string(),
                    requires_confirm: false,
                },
                PackageAction {
                    label: "App Data Usage".to_string(),
                    icon: "💾".to_string(),
                    description: "Show data storage used by an application".to_string(),
                    command: "adb shell dumpsys diskstats <package_name>".to_string(),
                    category: "Info".to_string(),
                    requires_confirm: false,
                },
                PackageAction {
                    label: "Force Stop App".to_string(),
                    icon: "⏹️".to_string(),
                    description: "Force stop a running application".to_string(),
                    command: "adb shell am force-stop <package_name>".to_string(),
                    category: "Manage".to_string(),
                    requires_confirm: false,
                },
            ],
            should_quit: false,
            show_help: false,
        }
    }

    fn next(&mut self) {
        if self.selected < self.actions.len() - 1 {
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

    fn get_selected_action(&self) -> &PackageAction {
        &self.actions[self.selected]
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
                        let action = app.get_selected_action();
                        if action.requires_confirm {
                            eprintln!("Would execute (requires confirmation): {}", action.command);
                        } else {
                            eprintln!("Would execute: {}", action.command);
                        }
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
        render_actions(frame, chunks[1], app);
    }

    // Render footer
    render_footer(frame, chunks[2], app.show_help);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled("📦 ", Style::default().fg(ANDROID_GREEN)),
        Span::styled(
            "Package Manager ",
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

fn render_actions(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Render action list
    let items: Vec<ListItem> = app
        .actions
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let style = if i == app.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(ANDROID_GREEN)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let warning = if action.requires_confirm {
                " ⚠️"
            } else {
                ""
            };

            ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(&action.icon, Style::default()),
                Span::raw(" "),
                Span::styled(&action.label, style),
                Span::styled(warning, Style::default().fg(Color::Yellow)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Package Actions ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ANDROID_GREEN)),
    );

    frame.render_widget(list, chunks[0]);

    // Render details panel
    let selected = &app.actions[app.selected];

    let category_color = match selected.category.as_str() {
        "Query" => Color::Cyan,
        "Info" => Color::Blue,
        "Manage" => Color::Yellow,
        "Permissions" => Color::Magenta,
        _ => Color::White,
    };

    let mut details_lines = vec![
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
            Span::styled(&selected.category, Style::default().fg(category_color)),
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
    ];

    if selected.requires_confirm {
        details_lines.push(Line::from(""));
        details_lines.push(Line::from(""));
        details_lines.push(Line::from(vec![Span::styled(
            "⚠️  Warning: Requires Confirmation",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )]));
        details_lines.push(Line::from(vec![Span::styled(
            "This action cannot be easily undone",
            Style::default().fg(Color::Gray),
        )]));
    } else {
        details_lines.push(Line::from(""));
        details_lines.push(Line::from(""));
        details_lines.push(Line::from(vec![Span::styled(
            "💡 Tip:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        details_lines.push(Line::from(vec![Span::styled(
            "Press Enter to execute this action",
            Style::default().fg(Color::Gray),
        )]));
    }

    let details = Paragraph::new(details_lines)
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
            "Package Manager",
            Style::default()
                .fg(ANDROID_GREEN)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("Manage Android packages (apps) on your device using ADB."),
        Line::from("Query information, install/uninstall apps, manage permissions, and more."),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Action Categories:",
            Style::default()
                .fg(ANDROID_GREEN)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Query", Style::default().fg(Color::Cyan)),
            Span::raw("       - List and search packages"),
        ]),
        Line::from(vec![
            Span::styled("  Info", Style::default().fg(Color::Blue)),
            Span::raw("        - Get detailed package information"),
        ]),
        Line::from(vec![
            Span::styled("  Manage", Style::default().fg(Color::Yellow)),
            Span::raw("      - Install, uninstall, enable/disable apps"),
        ]),
        Line::from(vec![
            Span::styled("  Permissions", Style::default().fg(Color::Magenta)),
            Span::raw(" - Grant or revoke app permissions"),
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
            Span::raw("   - Execute selected action"),
        ]),
        Line::from(vec![
            Span::styled("  h/?", Style::default().fg(ANDROID_GREEN)),
            Span::raw("          - Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("  q/Esc", Style::default().fg(ANDROID_GREEN)),
            Span::raw("        - Quit application"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "⚠️  Safety:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("Actions marked with ⚠️  require confirmation as they can"),
        Line::from("modify or delete data. Always double-check before confirming."),
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
        "↑/↓: Navigate | Enter/Space: Execute | h/?: Help | q/Esc: Quit"
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
