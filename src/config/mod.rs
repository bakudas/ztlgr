mod settings;
mod theme;

pub use settings::Config;
pub use theme::Theme;

pub mod themes {
    pub use super::theme::dracula::DraculaTheme;
    pub use super::theme::gruvbox::GruvboxTheme;
    pub use super::theme::nord::NordTheme;
    pub use super::theme::solarized::SolarizedTheme;
}
