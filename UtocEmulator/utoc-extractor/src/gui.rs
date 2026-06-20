use eframe::Frame;
use egui::Ui;
use egui_dock::{DockArea, DockState, Style};
use crate::actions::convert::ConvertAction;
use crate::actions::unpack::UnpackAction;
use crate::common::{get_config, get_egui_state, AppTabView};
use crate::GenericResult;

pub trait AppAction {
    fn ui(&mut self, ui: &mut Ui);
}

pub struct App {
    tabs: DockState<AppTab>
}

pub struct AppTab {
    pub(crate) title: String,
    pub(crate) contents: Option<Box<dyn AppAction>>
}

impl Default for App {
    fn default() -> Self {
        Self {
            // mode: AppMode::Extract
            tabs: DockState::new(vec![
                AppTab {
                    title: "Unpack".to_string(),
                    contents: Some(Box::new(UnpackAction::default()))
                },
                AppTab {
                    title: "Convert Metadata".to_string(),
                    contents: Some(Box::new(ConvertAction::default()))
                },
            ])
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _: &mut Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            DockArea::new(&mut self.tabs)
                .style(Style::from_egui(ui.style().as_ref()))
                .show_close_buttons(false)
                .show_leaf_collapse_buttons(false)
                .show_leaf_close_all_buttons(false)
                .show_inside(ui, &mut AppTabView);
        });
    }
}

pub fn execute() -> GenericResult<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([720., 720.]),
        ..Default::default()
    };
    get_config()?;
    get_egui_state()?;
    eframe::run_native(
        "Unreal Essentials UTOC Extractor", options,
        Box::new(|_| {
            Ok(Box::<App>::default())
        })
    )?;
    Ok(())
}