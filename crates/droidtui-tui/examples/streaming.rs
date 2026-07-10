//! Screen Streaming Example
//!
//! This example demonstrates the screen streaming capabilities of droidtui.
//! It shows how to set up and control Android device screen streaming.
//!
//! Run with: cargo run --example streaming

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

/// Streaming option
struct StreamOption {
    label: String,
    description: String,
    command: String,
    quality: String,
}

/// Application state
struct App {
    selected: usize,
    options: Vec<StreamOption>,
    should_quit: bool,
    show_help: bool,
    is_streaming: bool,
}

impl App {
    fn new() -> Self {
        Self {
            selected: 0,
            options: vec![
                StreamOption {
                    label: "📺 Standard Quality".to_string(),
                    description: "Stream at standard quality (720p, 2Mbps) - Balanced performance"
                        .to_string(),
                    command: "scrcpy --max-size 720 --bit-rate 2M".to_string(),
                    quality: "720p @ 2Mbps".to_string(),
                },
                StreamOption {
                    label: "🎬 High Quality".to_string(),
                    description: "Stream at high quality (1080p, 8Mbps) - Best visual quality"
                        .to_string(),
                    command: "scrcpy --max-size 1080 --bit-rate 8M".to_string(),
                    quality: "1080p @ 8Mbps".to_string(),
                },
                StreamOption {
                    label: "⚡ Low Latency".to_string(),
                    description: "Stream with minimal latency (480p, 1Mbps) - Fastest response"
                        .to_string(),
                    command: "scrcpy --max-size 480 --bit-rate 1M --max-fps 60".to_string(),
                    quality: "480p @ 1Mbps".to_string(),
                },
                StreamOption {
                    label: "🎮 Gaming Mode".to_string(),
                    description: "Optimized for gaming (720p, 4Mbps, 60fps) - High frame rate"
                        .to_string(),
                    command: "scrcpy --max-size 720 --bit-rate 4M --max-fps 60".to_string(),
                    quality: "720p @ 4Mbps 60fps".to_string(),
                },
                StreamOption {
                    label: "📱 Full Resolution".to_string(),
                    description: "Stream at device native resolution - Maximum detail".to_string(),
                    command: "scrcpy --bit-rate 16M".to_string(),
                    quality: "Native @ 16Mbps".to_string(),
                },
                StreamOption {
                    label: "🔒 Read-Only Mode".to_string(),
                    description: "View only, no control (720p, 2Mbps) - Safe observation"
                        .to_string(),
                    command: "scrcpy --max-size 720 --bit-rate 2M --no-control".to_string(),
                    quality: "720p @ 2Mbps (No Control)".to_string(),
                },
                StreamOption {
                    label: "🖼️ Borderless Window".to_string(),
                    description: "Stream in borderless window (1080p, 8Mbps) - Clean interface"
                        .to_string(),
                    command: "scrcpy --max-size 1080 --bit-rate 8M --window-borderless".to_string(),
                    quality: "1080p @ 8Mbps".to_string(),
                },
                StreamOption {
                    label: "🎯 Always On Top".to_string(),
                    description: "Keep stream window always visible (720p, 2Mbps)".to_string(),
                    command: "scrcpy --max-size 720 --bit-rate 2M --always-on-top".to_string(),
                    quality: "720p @ 2Mbps".to_string(),
                },
                StreamOption {
                    label: "📹 Record Session".to_string(),
                    description: "Stream and record to file (1080p, 8Mbps) - Save video"
                        .to_string(),
                    command: "scrcpy --max-size 1080 --bit-rate 8M --record session.mp4"
                        .to_string(),
                    quality: "1080p @ 8Mbps + Recording".to_string(),
                },
                StreamOption {
                    label: "🔇 No Audio".to_string(),
                    description: "Stream video only, no audio (720p, 2Mbps) - Silent mode"
                        .to_string(),
                    command: "scrcpy --max-size 720 --bit-rate 2M --no-audio".to_string(),
                    quality: "720p @ 2Mbps (No Audio)".to_string(),
                },
            ],
            should_quit: false,
            show_help: false,
            is_streaming: false,
        }
    }

    fn next(&mut self) {
        if self.selected < self.options.len() - 1 {
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

    fn get_selected_option(&self) -> &StreamOption {
        &self.options[self.selected]
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
                        let command = app.get_selected_option().command.clone();
                        app.is_streaming = !app.is_streaming;
                        eprintln!("Would execute: {}", command);
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
        render_streaming_options(frame, chunks[1], app);
    }

    // Render footer
    render_footer(frame, chunks[2], app);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled("📺 ", Style::default().fg(ANDROID_GREEN)),
        Span::styled(
            "Screen Streaming ",
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

fn render_streaming_options(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    // Render options list
    let items: Vec<ListItem> = app
        .options
        .iter()
        .enumerate()
        .map(|(i, option)| {
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
                Span::styled(&option.label, style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Streaming Options ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ANDROID_GREEN)),
    );

    frame.render_widget(list, chunks[0]);

    // Render details panel
    let selected = &app.options[app.selected];
    let details = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "Quality:",
            Style::default()
                .fg(ANDROID_GREEN)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            &selected.quality,
            Style::default().fg(Color::Cyan),
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
        Line::from(vec![Span::styled(
            "Note:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "Requires scrcpy to be installed",
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
            "Screen Streaming with scrcpy",
            Style::default()
                .fg(ANDROID_GREEN)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("DroidTUI uses scrcpy for screen streaming, which provides:"),
        Line::from("  • Low latency (35-70ms)"),
        Line::from("  • High quality (up to 1920×1080)"),
        Line::from("  • Mouse and keyboard control"),
        Line::from("  • Copy/paste between device and computer"),
        Line::from("  • No root required"),
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
            Span::raw("        - Move up"),
        ]),
        Line::from(vec![
            Span::styled("  ↓/j", Style::default().fg(ANDROID_GREEN)),
            Span::raw("        - Move down"),
        ]),
        Line::from(vec![
            Span::styled("  Enter/Space", Style::default().fg(ANDROID_GREEN)),
            Span::raw(" - Start streaming with selected option"),
        ]),
        Line::from(vec![
            Span::styled("  h/?", Style::default().fg(ANDROID_GREEN)),
            Span::raw("        - Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("  q/Esc", Style::default().fg(ANDROID_GREEN)),
            Span::raw("      - Quit"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Installation:",
            Style::default()
                .fg(ANDROID_GREEN)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("  macOS:     brew install scrcpy"),
        Line::from("  Linux:     sudo apt install scrcpy"),
        Line::from("  Windows:   scoop install scrcpy"),
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

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let footer_text = if app.show_help {
        "Press h/? to return | q/Esc to quit"
    } else if app.is_streaming {
        "🔴 Streaming Active | Press Enter to stop | q/Esc: Quit"
    } else {
        "↑/↓: Navigate | Enter/Space: Start Stream | h/?: Help | q/Esc: Quit"
    };

    let style = if app.is_streaming {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let footer = Paragraph::new(Line::from(vec![Span::styled(footer_text, style)]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}
