//! Top button bar.
//!
//! Left-aligned action cluster (Load, Save, …) and, right-aligned, a
//! theme-toggle pill followed by the Settings button. The save-game metadata that used to live here
//! now lives in the window title (see `AppState::window_title`), so
//! the bar is purely actions.

use eframe::egui;
use egui_components::Button;
use egui_components::theme::{Theme, ThemeMode};

use crate::settings::ThemeSetting;

/// Click signals emitted by the header bar this frame. Booleans
/// keep the host side trivial (`if header_action.save_clicked {
/// … }`) and let new buttons be added without revisiting the
/// caller's match arm.
#[derive(Default, Clone, Copy)]
pub struct HeaderAction {
    pub load_clicked: bool,
    pub save_clicked: bool,
    pub items_clicked: bool,
    pub spells_clicked: bool,
    pub settings_clicked: bool,
    /// The theme the toggle just switched to (already installed).
    pub theme_changed: Option<ThemeSetting>,
}

pub struct HeaderPanel;

impl HeaderPanel {
    /// Paint the button bar and report which buttons were clicked
    /// this frame. The game's buttons (Load, Save, Items, Spells) are
    /// disabled until `game_loaded`.
    pub fn show(&self, ui: &mut egui::Ui, game_loaded: bool) -> HeaderAction {
        let theme = Theme::get(ui.ctx());
        let mut action = HeaderAction::default();
        egui::Panel::top("keeper_header")
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(theme.colors.muted_background)
                    .inner_margin(egui::Margin::symmetric(14, 10)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let game_button = |label| Button::primary(label).small().disabled(!game_loaded);
                    if ui.add(game_button("Load")).clicked() {
                        action.load_clicked = true;
                    }
                    if ui.add(game_button("Save")).clicked() {
                        action.save_clicked = true;
                    }
                    // Divider between the save-game actions (Load/Save) and
                    // the resource-browser windows (Items/Spells).
                    ui.separator();
                    if ui.add(game_button("Items")).clicked() {
                        action.items_clicked = true;
                    }
                    if ui.add(game_button("Spells")).clicked() {
                        action.spells_clicked = true;
                    }
                    // Right-to-left: Settings is laid out first so it sits
                    // rightmost, past the theme toggle.
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(Button::ghost("Settings").small()).clicked() {
                            action.settings_clicked = true;
                        }
                        action.theme_changed = theme_toggle_button(ui);
                    });
                });
            });
        action
    }
}

/// Install the egui theme matching `theme`.
pub fn install_theme(ctx: &egui::Context, theme: ThemeSetting) {
    match theme {
        ThemeSetting::Light => Theme::light(),
        ThemeSetting::Dark => Theme::dark(),
    }
    .install(ctx);
}

/// Pill that swaps the active theme between light and dark, returning
/// the theme it switched to. Label shows the *target* mode (same
/// convention as the explorer's left rail), so it reads as a switch
/// rather than a destination.
fn theme_toggle_button(ui: &mut egui::Ui) -> Option<ThemeSetting> {
    let (target_label, next) = match Theme::get(ui.ctx()).mode {
        ThemeMode::Dark => ("Light", ThemeSetting::Light),
        ThemeMode::Light => ("Dark", ThemeSetting::Dark),
    };
    if !ui.add(Button::ghost(target_label).small()).clicked() {
        return None;
    }
    install_theme(ui.ctx(), next);
    Some(next)
}
