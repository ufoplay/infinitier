mod app;
mod build_data;
mod components;
mod game_view;
mod settings;
mod state;
mod ui;

use std::path::PathBuf;

use clap::Parser;
use eframe::egui;

use crate::app::KeeperApp;
use crate::build_data::build_version;
use crate::settings::SETTINGS_FILE;
use crate::state::app_title;
use crate::ui::{SettingsAction, install_theme};

/// Infinitier Keeper — cross-engine Infinity Engine save-game editor.
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Settings file holding the games listed in the Settings dialog.
    /// Created on the first change if it doesn't exist.
    #[arg(long, default_value = SETTINGS_FILE)]
    config: PathBuf,
    /// Log filter, e.g. "warn", "debug", "infinitier=debug,warn".
    #[arg(long, default_value = "infinitier=debug,warn")]
    log: String,
}

fn main() {
    let args = Args::parse();
    env_logger::Builder::new().parse_filters(&args.log).init();

    log::info!("Infinitier Keeper {}", build_version());

    // No game is loaded at startup: the user picks one in Settings.
    let settings_action = SettingsAction::load(args.config);
    let title = app_title();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(&title)
            .with_clamp_size_to_monitor_size(true)
            .with_inner_size(egui::vec2(1400.0, 1050.0))
            .with_min_inner_size(egui::vec2(900.0, 540.0)),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        &title,
        options,
        Box::new(move |cc| {
            install_inter_font(&cc.egui_ctx);
            install_theme(&cc.egui_ctx, settings_action.theme());

            // Same Linux/X11 DPI workaround as the explorer crate.
            #[cfg(target_os = "linux")]
            {
                let is_x11 =
                    std::env::var("WAYLAND_DISPLAY").is_err() && std::env::var("DISPLAY").is_ok();
                if is_x11 {
                    if let Ok(scale) = std::env::var("INFINITIER_SCALE") {
                        if let Ok(ppp) = scale.parse::<f32>() {
                            cc.egui_ctx.set_pixels_per_point(ppp);
                        }
                    } else {
                        let ppp = cc.egui_ctx.pixels_per_point();
                        if ppp > 1.5 {
                            cc.egui_ctx.set_pixels_per_point(1.5);
                        }
                    }
                }
            }

            Ok(Box::new(KeeperApp::new(settings_action)))
        }),
    ) {
        log::error!("Failed to run keeper: {e}");
    }
}

/// Replace egui's default proportional font (`Ubuntu-Light`, which
/// looks washed out on a light background) with Inter — the same
/// typeface shadcn / gpui-component use. Bytes come from the
/// `ttf-inter` crate, so we're not adding any binary assets to this
/// repo. egui's built-in fallback chain (emoji, monospace, the
/// original Ubuntu-Light) stays in place for any glyph Inter
/// doesn't cover.
fn install_inter_font(ctx: &egui::Context) {
    use std::sync::Arc;

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "Inter".to_owned(),
        Arc::new(egui::FontData::from_static(ttf_inter::REGULAR)),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "Inter".to_owned());
    ctx.set_fonts(fonts);
}
