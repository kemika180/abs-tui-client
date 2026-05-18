use ratatui::style::Color;

pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub border: Color,
    pub highlight: Color,
}

impl Theme {
    pub fn tokyo_night() -> Self {
        Self {
            bg: Color::Rgb(26, 27, 38),
            fg: Color::Rgb(169, 177, 214),
            selection_bg: Color::Rgb(51, 71, 110),
            selection_fg: Color::Rgb(192, 202, 245),
            border: Color::Rgb(86, 95, 137),
            highlight: Color::Rgb(122, 162, 247),
        }
    }
}
