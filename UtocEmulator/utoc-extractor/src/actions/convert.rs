use std::fs::{File, ReadDir};
use std::io::BufWriter;
use std::path::{Component, Path, PathBuf};
use std::time::Instant;
use anyhow::anyhow;
use chrono::Utc;
use egui::{Align, Button, ComboBox, Layout, RichText, TextStyle, Ui};
use egui_extras::{Column, TableBuilder};
use ini::Ini;
use retoc::ser::{WriteExt, Writeable};
use retoc::version::EngineVersion;
#[cfg(not(target_os = "windows"))]
use rfd::FileDialog;
use walkdir::{DirEntry, WalkDir};
#[cfg(target_os = "windows")]
use wfd::{ DialogParams, FOS_PICKFOLDERS };
use utoc_lib::assets::{asset_path_to_package_id, convert_to_asset_path, convert_to_package_id, UASSETMETA_EXTENSION, UASSET_EXTENSION, UMAP_EXTENSION, UTOCMETA};
use utoc_lib::metadata::UtocMetadata;
use crate::common::{convert_to_ue_path, get_config, get_default_directory, set_default_directory, try_get_user_config, ActionInfo, AssetMetadata, ConvertExecutor, FilePicker, FilterByAsset, UIComponent};
use crate::GenericResult;
use crate::gui::AppAction;

pub struct ConvertAction {
    input: FilePicker,
    info: Option<ActionInfo>,
    current_format: Option<AssetMetadata>,
    convert_to: AssetMetadata,
    engine_version: EngineVersion,
    asset_list: Vec<PathBuf>
}

const CONVERT_INPUT_TITLE: &'static str = "Select the UnrealEssentials folder in your mod";
const CONVERT_INPUT_KEY: &'static str = "convert_input";

impl ConvertAction {
    #[cfg(not(target_os = "windows"))]
    fn convert_input(mut ini: Option<Ini>) -> Option<PathBuf> {
        let dialog = FileDialog::new()
            .set_title(CONVERT_INPUT_TITLE);
        let path = get_default_directory(dialog, ini.as_mut(), CONVERT_INPUT_KEY)
            .pick_folder();
        if path.is_some() {
            set_default_directory(path.as_ref(), ini.as_mut(), CONVERT_INPUT_KEY);
        }
        path
    }

    #[cfg(target_os = "windows")]
    fn convert_input(mut ini: Option<Ini>) -> Option<PathBuf> {
        let mut params = DialogParams {
            title: CONVERT_INPUT_TITLE,
            options: FOS_PICKFOLDERS,
            ..Default::default()
        };
        if let Some(folder) = get_default_directory(ini.as_mut(), CONVERT_INPUT_KEY) {
            params.folder = folder;
        }
        let result = wfd::open_dialog(params).map(|v| v.selected_file_path).ok();
        if result.is_some() {
            set_default_directory(Some(result.as_ref()?), ini.as_mut(), CONVERT_INPUT_KEY);
        }
        result
    }
}

impl Default for ConvertAction {
    fn default() -> Self {
        Self {
            input: FilePicker::new("Folder Containing Assets: ", Self::convert_input),
            info: None,
            current_format: None,
            convert_to: AssetMetadata::Table,
            engine_version: EngineVersion::UE5_3,
            asset_list: vec![]
        }
    }
}

impl ConvertAction {
    pub fn select_mod_folder(&mut self) -> GenericResult<()> {
        self.asset_list.clear();
        self.current_format = None;
        self.asset_list = WalkDir::new(self.input.get_path()).into_iter()
            .filter_map(|d| FilterByAsset::filter_by_asset_path(self.input.get_path(), d)).collect();
        // Metadata check
        let has_toc_meta = std::fs::read_dir(self.input.get_path())?
            .find(FilterByAsset::check_utocmeta).is_some();
        let asset_meta: Vec<_> = self.asset_list.iter().filter(|v| {
            std::fs::exists(self.input.get_path().join(v).with_extension(UASSETMETA_EXTENSION)).unwrap()
        }).collect();
        let no_meta = !has_toc_meta && asset_meta.is_empty();
        if no_meta && self.engine_version < EngineVersion::UE5_3 {
            return Err(anyhow!("No asset metadata exists in this mod.").into_boxed_dyn_error());
        }
        let both_meta = has_toc_meta && !asset_meta.is_empty();
        if both_meta {
            return Err(anyhow!("Expected the mod to only have one type of asset metadata.").into_boxed_dyn_error());
        }
        if !asset_meta.is_empty() && self.asset_list.len() != asset_meta.len() {
            return Err(anyhow!("Expected every asset to have an associated .uassetmeta.").into_boxed_dyn_error());
        }
        // Setup loaded state
        if has_toc_meta {
            self.current_format = Some(AssetMetadata::Table);
        } else if !asset_meta.is_empty() {
            self.current_format = Some(AssetMetadata::PerAsset);
        }
        self.check_convert_type_same();
        Ok(())
    }

    fn check_convert_type_same(&mut self) {
        if let Some(fmt) = self.current_format && fmt == self.convert_to {
            self.info = Some(ActionInfo::info("The convert to option is the same as the mod's current metadata!".to_string()));
        } else {
            self.info = None;
        }
    }

    fn metadata_convert_option_selectable(&mut self, ui: &mut Ui, opt: AssetMetadata) {
        if ui.selectable_value(&mut self.convert_to, opt, format!("{}", opt)).clicked() {
            self.check_convert_type_same();
        }
    }

    fn get_game_name_from_folder(&self) -> Option<String> {
        for asset in &self.asset_list {
            let components: Vec<_> = asset.components().collect();
            match components.first().unwrap() {
                Component::Normal(part) => {
                    let part = part.to_str().unwrap();
                    if part != "Engine" {
                        return Some(part.to_string());
                    }
                },
                _ => continue
            }
        }
        None
    }

    pub fn convert_to(&mut self) -> GenericResult<()> {
        ConvertExecutor::convert(
            self.input.get_path(),
            self.current_format.unwrap(),
            self.convert_to,
            self.asset_list.as_slice(),
            self.engine_version
        )?;
        self.current_format = Some(match self.convert_to {
            AssetMetadata::None => AssetMetadata::None,
            AssetMetadata::PerAsset => AssetMetadata::Table,
            AssetMetadata::Table => AssetMetadata::PerAsset
        });
        Ok(())
    }

    fn adjust_settings_by_user_config(&mut self) -> Result<(), ActionInfo> {
        if let Err(e) = self.select_mod_folder() {
            self.asset_list.clear();
            return Err(ActionInfo::error(format!("Can not use folder: {}", e.to_string())));
        }
        let conf = try_get_user_config();
        if let Err(e) = &conf {
            return Err(ActionInfo::info(
                format!("Could not open config.ini: {}", e.to_string())));
        }
        let conf = conf.unwrap();
        let versions = conf.section(Some("ProjectVersions"));
        if versions.is_none() {
            return Err(ActionInfo::info(
                "Section \"ProjectVersions\" is missing from your config.ini!".to_string()));
        }
        let versions = versions.unwrap();
        if let Some(game_name) = self.get_game_name_from_folder() &&
            let Some(value) = versions.get(game_name.as_str()) {
            match match value {
                "UE4_25" => Some(EngineVersion::UE4_25),
                "UE4_26" => Some(EngineVersion::UE4_26),
                "UE4_27" => Some(EngineVersion::UE4_27),
                "UE5_0" => Some(EngineVersion::UE5_0),
                "UE5_1" => Some(EngineVersion::UE5_1),
                "UE5_2" => Some(EngineVersion::UE5_2),
                "UE5_3" => Some(EngineVersion::UE5_3),
                "UE5_4" => Some(EngineVersion::UE5_4),
                "UE5_5" => Some(EngineVersion::UE5_5),
                "UE5_6" => Some(EngineVersion::UE5_6),
                "UE5_7" => Some(EngineVersion::UE5_7),
                _ => None
            } {
                Some(v) => self.engine_version = v,
                None => return Err(ActionInfo::info(
                    format!("Engine version \"{}\" does not exist.", value)))
            };
        }
        Ok(())
    }
}

impl AppAction for ConvertAction {
    fn ui(&mut self, ui: &mut Ui) {
        if let Some(info) = &self.info {
            ui.label(RichText::new(&info.text)
                .color(info.color));
        }
        if self.input.ui(ui) {
            if let Err(e) = self.adjust_settings_by_user_config() {
                self.info = Some(e);
            }
        }
        ui.horizontal(|ui| {
            if self.engine_version < EngineVersion::UE5_3 && self.convert_to == AssetMetadata::None {
                self.convert_to = AssetMetadata::Table;
            }
            ui.label("Convert To: ");
            ComboBox::new("metadata_convert_to", "")
                .selected_text(format!("{}", self.convert_to))
                .show_ui(ui, |ui| {
                    if self.engine_version >= EngineVersion::UE5_3 {
                        self.metadata_convert_option_selectable(ui, AssetMetadata::None);
                    }
                    self.metadata_convert_option_selectable(ui, AssetMetadata::PerAsset);
                    self.metadata_convert_option_selectable(ui, AssetMetadata::Table);
                });
            ui.label("Engine Version:");
            UIComponent::engine_version_combobox(ui, &mut self.engine_version);
            if let Some(format) = self.current_format {
                ui.label(format!("Current format: {:?}", format));
            }
        });

        let remain = ui.max_rect().height() - ui.min_rect().height() - 48.;
        let text_height = TextStyle::Body.resolve(ui.style())
            .size.max(ui.spacing().interact_size.y);
        if self.asset_list.len() > 0 {
            ui.label(format!("{} assets", self.asset_list.len()));
            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::remainder())
                .min_scrolled_height(0.)
                .max_scroll_height(remain);
            table.body(|body| {
                body.rows(text_height, self.asset_list.len(), |mut row| {
                    let path = self.asset_list[row.index()].as_path();
                    row.col(|ui| {
                        ui.label(format!("{}", path.to_str().unwrap()));
                    });
                });
            });
        }
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            let enabled = self.asset_list.len() > 0 &&
            if let Some(fmt) = self.current_format && fmt != self.convert_to { true } else { false };
            ui.add_enabled_ui(enabled, |ui| {
                if ui.add(Button::new("Convert")
                    .min_size([ui.max_rect().width(), 32.].into()))
                    .clicked() {
                    let start = Instant::now();
                    self.info = match self.convert_to() {
                        Ok(_) => {
                            self.check_convert_type_same();
                            let duration = Instant::now().duration_since(start)
                                .as_micros() as f64 / 1000.;
                            let time = Utc::now().time().format("%-I:%M:%S");
                            let fmt = format!(
                                "Converted {} assets! (Took {} ms at {})", self.asset_list.len(), duration, time);
                            Some(ActionInfo::info(fmt))
                        },
                        Err(e) => Some(ActionInfo::error(format!("Failed to convert assets: {}", e.to_string())))
                    }
                }
            });
        });
    }
}