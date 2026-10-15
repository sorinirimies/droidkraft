//! Semantic styles wrapping [`tui_nuances`]. `t` cycles themes.

use ratatui::style::{Color, Modifier, Style};
use tui_nuances::{Status, Theme, ThemeSwitcher};

pub struct Themes {
    sw: ThemeSwitcher,
}

impl Default for Themes {
    fn default() -> Self {
        let mut sw = ThemeSwitcher::new();
        sw.set_by_name("porsche");
        Self { sw }
    }
}

impl Themes {
    pub fn set_by_name(&mut self, name: &str) {
        self.sw.set_by_name(name);
    }
    pub fn cycle(&mut self) {
        self.sw.next();
    }
    pub fn name(&self) -> &str {
        &self.sw.current().name
    }
    pub fn index(&self) -> usize {
        self.sw.index()
    }
    pub fn raw(&self) -> &Theme {
        self.sw.current()
    }
    pub fn base(&self) -> Style {
        self.raw().style()
    }
    pub fn muted(&self) -> Style {
        self.raw().muted_style()
    }
    pub fn title(&self) -> Style {
        self.raw().title_style().add_modifier(Modifier::BOLD)
    }
    pub fn border(&self, focused: bool) -> Style {
        self.raw().border_style(focused)
    }
    pub fn selected(&self) -> Style {
        self.raw().selected_style()
    }
    pub fn accent(&self) -> Color {
        self.raw().accent
    }
    pub fn primary(&self) -> Color {
        self.raw().primary
    }
    pub fn status(&self, s: Status) -> Style {
        self.raw().status_style(s)
    }
    pub fn status_color(&self, s: Status) -> Color {
        self.raw().status_color(s)
    }
    pub fn fg(&self) -> Color {
        self.raw().foreground
    }
}
