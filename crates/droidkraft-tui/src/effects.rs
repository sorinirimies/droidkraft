use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Paragraph, Widget},
};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct EffectsManager {
    pub start_time: Instant,
    pub startup_duration: Duration,
    pub tick_count: u64,
    pub fade_in_start: Option<Instant>,
    pub fade_in_duration: Duration,
    pub slide_in_start: Option<Instant>,
    pub slide_in_duration: Duration,
    pub slide_out_start: Option<Instant>,
    pub slide_out_duration: Duration,
}

impl EffectsManager {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            startup_duration: Duration::from_millis(2500),
            tick_count: 0,
            fade_in_start: None,
            fade_in_duration: Duration::from_millis(300),
            slide_in_start: None,
            slide_in_duration: Duration::from_millis(250),
            slide_out_start: None,
            slide_out_duration: Duration::from_millis(200),
        }
    }

    pub fn tick(&mut self, _elapsed: Duration) {
        self.tick_count += 1;
    }

    pub fn start_fade_in(&mut self) {
        self.fade_in_start = Some(Instant::now());
    }

    pub fn get_fade_in_progress(&self) -> f32 {
        if let Some(start) = self.fade_in_start {
            let elapsed = start.elapsed();
            if elapsed >= self.fade_in_duration {
                1.0
            } else {
                elapsed.as_millis() as f32 / self.fade_in_duration.as_millis() as f32
            }
        } else {
            1.0
        }
    }

    pub fn is_fade_in_complete(&self) -> bool {
        if let Some(start) = self.fade_in_start {
            start.elapsed() >= self.fade_in_duration
        } else {
            true
        }
    }

    pub fn is_startup_complete(&self) -> bool {
        self.start_time.elapsed() >= self.startup_duration
    }

    pub fn start_slide_in(&mut self) {
        self.slide_in_start = Some(Instant::now());
    }

    pub fn start_slide_out(&mut self) {
        self.slide_out_start = Some(Instant::now());
    }

    pub fn get_slide_in_progress(&self) -> f32 {
        if let Some(start) = self.slide_in_start {
            let elapsed = start.elapsed();
            if elapsed >= self.slide_in_duration {
                1.0
            } else {
                let progress =
                    elapsed.as_millis() as f32 / self.slide_in_duration.as_millis() as f32;
                // Ease out cubic for smooth deceleration
                1.0 - (1.0 - progress).powi(3)
            }
        } else {
            1.0
        }
    }

    pub fn get_slide_out_progress(&self) -> f32 {
        if let Some(start) = self.slide_out_start {
            let elapsed = start.elapsed();
            if elapsed >= self.slide_out_duration {
                1.0
            } else {
                let progress =
                    elapsed.as_millis() as f32 / self.slide_out_duration.as_millis() as f32;
                // Ease in cubic for smooth acceleration
                progress.powi(3)
            }
        } else {
            0.0
        }
    }

    pub fn is_slide_in_complete(&self) -> bool {
        if let Some(start) = self.slide_in_start {
            start.elapsed() >= self.slide_in_duration
        } else {
            true
        }
    }

    pub fn is_slide_out_complete(&self) -> bool {
        if let Some(start) = self.slide_out_start {
            start.elapsed() >= self.slide_out_duration
        } else {
            false
        }
    }

    pub fn reset_slide(&mut self) {
        self.slide_in_start = None;
        self.slide_out_start = None;
    }

    pub fn get_startup_progress(&self) -> f32 {
        let elapsed = self.start_time.elapsed();
        if elapsed >= self.startup_duration {
            1.0
        } else {
            elapsed.as_millis() as f32 / self.startup_duration.as_millis() as f32
        }
    }

    pub fn get_reveal_alpha(&self) -> u8 {
        let progress = self.get_startup_progress();
        (progress * 255.0) as u8
    }

    pub fn get_wave_effect(&self) -> f32 {
        let time = self.tick_count as f32 * 0.1;
        (time.sin() + 1.0) / 2.0
    }
}

impl Default for EffectsManager {
    fn default() -> Self {
        Self::new()
    }
}

// Widget for the startup reveal animation
pub struct RevealWidget<'a> {
    effects_manager: &'a mut EffectsManager,
    subtitle: &'a str,
}

impl<'a> RevealWidget<'a> {
    pub fn new(
        effects_manager: &'a mut EffectsManager,
        _title: &'a str,
        subtitle: &'a str,
    ) -> Self {
        Self {
            effects_manager,
            subtitle,
        }
    }
}

impl<'a> Widget for RevealWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let progress = self.effects_manager.get_startup_progress();
        let alpha = self.effects_manager.get_reveal_alpha();
        let wave = self.effects_manager.get_wave_effect();

        // Create animated gradient background
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let distance_from_center = {
                    let center_x = area.width / 2;
                    let center_y = area.height / 2;
                    let dx = (x - area.left()).abs_diff(center_x) as f32;
                    let dy = (y - area.top()).abs_diff(center_y) as f32;
                    (dx * dx + dy * dy).sqrt()
                };

                let wave_intensity = (wave * 32.0) as u8;
                let base_intensity =
                    ((1.0 - distance_from_center / (area.width as f32)) * alpha as f32) as u8;
                let final_intensity = base_intensity.saturating_add(wave_intensity);

                let color = if progress < 1.0 {
                    // Reveal animation - sweep from center
                    Color::Rgb(0, final_intensity / 4, 0)
                } else {
                    // Completed - gentle pulse
                    Color::Rgb(0, (final_intensity / 6).max(8), 0)
                };

                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_bg(color);
                }
            }
        }

        // ASCII Art for DroidKraft and Android logo
        let ascii_art = r#"
           ██████╗ ██████╗  ██████╗ ██╗██████╗ ████████╗██╗   ██╗██╗
           ██╔══██╗██╔══██╗██╔═══██╗██║██╔══██╗╚══██╔══╝██║   ██║██║
           ██║  ██║██████╔╝██║   ██║██║██║  ██║   ██║   ██║   ██║██║
           ██║  ██║██╔══██╗██║   ██║██║██║  ██║   ██║   ██║   ██║██║
           ██████╔╝██║  ██║╚██████╔╝██║██████╔╝   ██║   ╚██████╔╝██║
           ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝╚═════╝    ╚═╝    ╚═════╝ ╚═╝

              🤖 Android Development Toolkit 🤖
"#;

        // Create the main content with fade-in effect
        let content = if progress < 0.3 {
            // Early phase - just show dots
            "●●●\n\nInitializing...".to_string()
        } else if progress < 0.6 {
            // Mid phase - show ASCII art
            ascii_art.to_string()
        } else if progress < 1.0 {
            // Late phase - show ASCII art with subtitle
            format!("{}\n{}", ascii_art, self.subtitle)
        } else {
            // Complete - show all with instructions
            format!(
                "{}\n{}\n\n⚡ Press any key to continue...",
                ascii_art, self.subtitle
            )
        };

        // Calculate text color based on progress
        let text_color = if progress < 1.0 {
            Color::Rgb(0, alpha, 0)
        } else {
            // Pulse effect when complete
            let pulse = ((self.effects_manager.tick_count / 20) % 2) as u8;
            if pulse == 0 {
                Color::LightGreen
            } else {
                Color::Green
            }
        };

        let block = Block::bordered()
            .title("🌟 Welcome to DroidKraft")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(text_color));

        let paragraph = Paragraph::new(content)
            .block(block)
            .style(Style::default().fg(text_color))
            .alignment(Alignment::Center);

        // Center the content - larger area for ASCII art
        let popup_area = centered_rect(90, 85, area);
        paragraph.render(popup_area, buf);
    }
}

// Helper function to create centered rectangles
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

// Menu highlight effects with consistent green color
pub fn get_selection_color(_tick_count: u64, _position: usize) -> Color {
    // Consistent green color for all selections
    Color::Green
}

// Loading animation characters
pub fn get_loading_spinner(tick_count: u64) -> &'static str {
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let index = (tick_count / 8) % spinner_chars.len() as u64;
    spinner_chars[index as usize]
}

// Loading dots animation
pub fn get_loading_dots(tick_count: u64) -> String {
    let dots_count = ((tick_count / 20) % 4) as usize;
    let dots = ".".repeat(dots_count);
    format!("Loading{:<3}", dots)
}

// Progress bar animation
pub fn get_progress_bar(tick_count: u64, width: usize) -> String {
    let progress = ((tick_count / 5) % width as u64) as usize;
    let filled = "█".repeat(progress);
    let empty = "░".repeat(width.saturating_sub(progress));
    format!("[{}{}]", filled, empty)
}

// Enhanced selection effect with consistent green color
pub fn get_selection_color_with_boost(_tick_count: u64, _position: usize, _boost: u64) -> Color {
    // Always return consistent green color, no boost effects for line selection
    Color::Green
}

// Orbital spinner animation (circles around)
pub fn get_orbital_spinner(tick_count: u64) -> &'static str {
    let orbital_chars = ["◐", "◓", "◑", "◒"];
    let index = (tick_count / 5) % orbital_chars.len() as u64;
    orbital_chars[index as usize]
}

// Wave animation for loading screen
pub fn get_wave_animation(tick_count: u64) -> String {
    let wave_chars = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    let wave_length = 15;
    let mut wave = String::new();

    for i in 0..wave_length {
        let offset = (tick_count as i32 + i * 2) % (wave_chars.len() as i32 * 2);
        let index = if offset >= wave_chars.len() as i32 {
            (wave_chars.len() as i32 * 2 - offset - 1).max(0)
        } else {
            offset
        } as usize;

        wave.push_str(wave_chars[index.min(wave_chars.len() - 1)]);
    }

    wave
}

// Circular progress indicator
pub fn get_circular_progress(tick_count: u64) -> String {
    let segments = ["◜", "◝", "◞", "◟"];
    let index = (tick_count / 3) % segments.len() as u64;
    segments[index as usize].to_string()
}

// Dots orbit animation (dots rotating in a circle)
pub fn get_dots_orbit(tick_count: u64) -> String {
    let positions = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let index = (tick_count / 4) % positions.len() as u64;
    positions[index as usize].to_string()
}

// Particle effect - expanding dots
pub fn get_particle_effect(tick_count: u64) -> String {
    let cycle = (tick_count / 6) % 8;
    match cycle {
        0 => "·  ·  ·".to_string(),
        1 => " · · · ".to_string(),
        2 => "  ···  ".to_string(),
        3 => "  ███  ".to_string(),
        4 => " █████ ".to_string(),
        5 => "  ███  ".to_string(),
        6 => " · · · ".to_string(),
        _ => "·  ·  ·".to_string(),
    }
}

// Bouncing ball animation
pub fn get_bouncing_ball(tick_count: u64) -> String {
    let positions = [
        "●       ",
        " ●      ",
        "  ●     ",
        "   ●    ",
        "    ●   ",
        "   ●    ",
        "  ●     ",
        " ●      ",
    ];
    let index = (tick_count / 5) % positions.len() as u64;
    positions[index as usize].to_string()
}

// Shimmer effect for selected items
pub fn get_shimmer_intensity(tick_count: u64) -> f32 {
    let phase = (tick_count as f32 * 0.15).sin();
    (phase + 1.0) / 2.0 // Normalize to 0.0-1.0
}

// Get shimmer color for menu item highlight
pub fn get_shimmer_color(tick_count: u64, base_color: Color) -> Color {
    let intensity = get_shimmer_intensity(tick_count);
    let brightness = (200.0 + intensity * 55.0) as u8;

    match base_color {
        Color::Green => Color::Rgb(0, brightness, 0),
        Color::Yellow => Color::Rgb(brightness, brightness, 0),
        _ => base_color,
    }
}

// Slide animation easing function (ease out cubic)
pub fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

// Slide animation easing function (ease in cubic)
pub fn ease_in_cubic(t: f32) -> f32 {
    t.powi(3)
}

// Bounce effect for emphasis
pub fn get_bounce_offset(tick_count: u64, duration_ticks: u64) -> f32 {
    if tick_count >= duration_ticks {
        return 0.0;
    }

    let progress = tick_count as f32 / duration_ticks as f32;
    let bounce = (progress * std::f32::consts::PI * 2.0).sin() * (1.0 - progress);
    bounce * 3.0 // Scale the bounce effect
}
