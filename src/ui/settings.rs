use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::Frame;

use crate::state::App;

pub fn draw(_app: &App, _frame: &mut Frame) {
    // TODO: implement settings dialogue
}

pub fn handle_key(app: &mut App, code: KeyCode, _modifiers: KeyModifiers) {
    if code == KeyCode::Esc {
        app.show_settings = false;
    }
}
