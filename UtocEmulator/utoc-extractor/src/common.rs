use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufWriter;
use std::path::{Component, Path, PathBuf};
use clap::ValueEnum;
use eframe::epaint::Color32;
use egui::{ComboBox, Id, TextEdit, Ui, WidgetText};
use egui_dock::TabViewer;
use ini::Ini;
use retoc::ser::WriteExt;
use retoc::Toc;
use retoc::version::EngineVersion;
use walkdir::DirEntry;
#[cfg(not(target_os = "windows"))]
use rfd::FileDialog;
use utoc_lib::assets::{convert_to_package_id, UASSETMETA_EXTENSION, UASSET_EXTENSION, UMAP_EXTENSION, UTOCMETA};
use utoc_lib::metadata::UtocMetadata;
use crate::cli::Progress;
use crate::GenericResult;
use crate::gui::AppTab;

#[derive(ValueEnum, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum AssetMetadata {
    None,
    Table,
    PerAsset
}

impl Display for AssetMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::None => "None",
            Self::PerAsset => "Per Asset",
            Self::Table => "Table",
        })
    }
}

pub(crate) fn get_root_path<P: AsRef<Path>>(output: P, mount_point: &str, toc: &Toc, root_folder: &str) -> PathBuf {
    if mount_point.len() > utoc_lib::assets::MOUNT_POINT.len() {
        output.as_ref().join(&mount_point[utoc_lib::assets::MOUNT_POINT.len()..])
    } else {
        // Determine if we need to make the /Game/Content folders ourselves:
        let mut make_folders = true;
        for (file, _) in &toc.file_map {
            if let Some(s) = file.split_once('/') && s.0 == "Engine" {
                make_folders = false;
                break;
            }
        }
        match make_folders {
            true => output.as_ref().join(root_folder).join("Content"),
            false => output.as_ref().to_owned()
        }
    }
}

#[derive(Debug)]
pub struct FilePicker {
    pub(crate) label: &'static str,
    pub(crate) path: String,
    pub(crate) open_dialog: fn(Option<Ini>) -> Option<PathBuf>
}

impl FilePicker {
    fn try_create_dialog() -> GenericResult<Ini> {
        let state = get_egui_state()?;
        let conf = Ini::load_from_file(state)?;
        Ok(conf)
    }

    pub(crate) fn ui(&mut self, ui: &mut Ui) -> bool {
        let mut loaded = false;
        ui.horizontal(|ui| {
            let id_other_size = Id::new(format!("{}_other_size", self.label));
            let max_width = ui.max_rect().width();
            let last_width = ui.data(|data| data.get_temp(id_other_size)
                .unwrap_or(max_width));
            let target_width = max_width - last_width;
            ui.label(self.label);
            ui.add(TextEdit::singleline(&mut self.path)
                .interactive(false)
                .desired_width(target_width));
            if ui.button("Select").clicked() {
                match Self::try_create_dialog() {
                    Ok(path) => if let Some(path) = (self.open_dialog)(Some(path)) {
                        self.path = path.to_str().unwrap().to_string();
                        loaded = true;
                    },
                    Err(e) => {
                        println!("{}: {}", console::style("Error").red(), e.to_string());
                    }
                }
            }
            ui.data_mut(|data| data.insert_temp(
                id_other_size,
                ui.min_rect().width() - target_width
            ));
        });
        loaded
    }
}

impl FilePicker {
    pub fn new(label: &'static str, open_dialog: fn(Option<Ini>) -> Option<PathBuf>) -> Self {
        Self { label, path: String::new(), open_dialog }
    }

    pub fn get_path(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }
}

pub struct AppTabView;
impl TabViewer for AppTabView {
    type Tab = AppTab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.title.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        if let Some(inner) = tab.contents.as_mut() {
            ui.vertical(|ui| {
                inner.ui(ui);
            });
        } else {
            ui.label("TODO");
        }
    }
}

pub struct ActionInfo {
    pub(crate) text: String,
    pub(crate) color: Color32
}

impl ActionInfo {
    pub fn error(text: String) -> Self {
        Self {
            text,
            color: Color32::from_rgb(255, 0, 0)
        }
    }

    pub fn info(text: String) -> Self {
        Self {
            text,
            color: Color32::from_rgb(255, 200, 0)
        }
    }
}

pub struct UIComponent;
impl UIComponent {
    pub(crate) fn engine_version_combobox(ui: &mut Ui, version: &mut EngineVersion) {
        ComboBox::new("engine_version", "")
            .selected_text(format!("{:?}", *version))
            .show_ui(ui, |ui| {
                ui.selectable_value(version, EngineVersion::UE4_25, format!("{:?}", EngineVersion::UE4_25));
                ui.selectable_value(version, EngineVersion::UE4_26, format!("{:?}", EngineVersion::UE4_26));
                ui.selectable_value(version, EngineVersion::UE4_27, format!("{:?}", EngineVersion::UE4_27));
                ui.selectable_value(version, EngineVersion::UE5_0, format!("{:?}", EngineVersion::UE5_0));
                ui.selectable_value(version, EngineVersion::UE5_1, format!("{:?}", EngineVersion::UE5_1));
                ui.selectable_value(version, EngineVersion::UE5_2, format!("{:?}", EngineVersion::UE5_2));
                ui.selectable_value(version, EngineVersion::UE5_3, format!("{:?}", EngineVersion::UE5_3));
                ui.selectable_value(version, EngineVersion::UE5_4, format!("{:?}", EngineVersion::UE5_4));
                ui.selectable_value(version, EngineVersion::UE5_5, format!("{:?}", EngineVersion::UE5_5));
                ui.selectable_value(version, EngineVersion::UE5_6, format!("{:?}", EngineVersion::UE5_6));
                ui.selectable_value(version, EngineVersion::UE5_7, format!("{:?}", EngineVersion::UE5_7));
            });
    }
}

pub fn get_config() -> GenericResult<PathBuf> {
    let exec = std::env::current_exe()?;
    let dir = exec.parent().unwrap();
    let path = dir.join("config.ini");
    if !std::fs::exists(path.as_path())? {
        let mut ini = Ini::new();
        _ = ini.with_section(Some("ProjectVersions"));
        ini.write_to_file(path.as_path())?;
    }
    Ok(path)
}


pub fn get_egui_state() -> GenericResult<PathBuf> {
    let exec = std::env::current_exe()?;
    let dir = exec.parent().unwrap();
    let path = dir.join("egui.ini");
    if !std::fs::exists(path.as_path())? {
        File::create(path.as_path())?;
    }
    Ok(path)
}

#[cfg(not(target_os = "windows"))]
pub fn get_default_directory(dialog: FileDialog, ini: Option<&mut Ini>, field: &str) -> FileDialog {
    if let Some(config) = ini &&
        let Some(input) = config.general_section().get(field) {
        dialog.set_directory(PathBuf::from(input))
    } else {
        dialog
    }
}

pub fn get_default_directory<'a>(ini: Option<&'a mut Ini>, field: &'a str) -> Option<&'a str> {
    ini.and_then(|config| config.general_section().get(field))
}

pub fn set_default_directory<P: AsRef<Path>>(path: Option<P>, ini: Option<&mut Ini>, field: &str) {
    if let Some(config) = ini && let Some(path) = path {
        let path = path.as_ref();
        config.with_general_section().set(field, path.to_str().unwrap());
            config.write_to_file(get_egui_state().unwrap()).unwrap();
    }
}

pub fn try_get_user_config() -> GenericResult<Ini> {
    let state = get_config()?;
    let conf = Ini::load_from_file(state)?;
    Ok(conf)
}

pub fn convert_to_ue_path<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().components().filter_map(|c| match c {
        Component::Normal(c) => Some(c.to_str().unwrap().to_string()),
        _ => None }).collect::<Vec<String>>().join("/")
}

static ASSET_EXTENSIONS: [&'static str; 2] = [
    UASSET_EXTENSION,
    UMAP_EXTENSION
];

pub struct FilterByAsset;
impl FilterByAsset {
    pub fn filter_by_asset(dir_entry: walkdir::Result<DirEntry>) -> Option<DirEntry> {
        dir_entry.ok()
            .and_then(|d| {
                let is_file = d.metadata().ok().map_or(false, |m| m.is_file());
                let check_ext = d.path().extension().map_or(
                    false, |ext| ASSET_EXTENSIONS.contains(&ext.to_str().unwrap()));
                if !is_file || !check_ext { return None }
                Some(d)
            })
    }

    pub fn filter_by_asset_path<P: AsRef<Path>>(root: P, dir_entry: walkdir::Result<DirEntry>) -> Option<PathBuf> {
        Self::filter_by_asset(dir_entry).map(|v| v.into_path()
            .strip_prefix(root.as_ref()).unwrap().to_owned())
    }

    pub fn check_utocmeta(dir_entry: &std::io::Result<std::fs::DirEntry>) -> bool {
        dir_entry.as_ref()
            .map_or(false, |d| {
                let is_file = d.metadata().ok().map_or(false, |m| m.is_file());
                is_file && d.file_name().to_str().unwrap() == UTOCMETA
            })
    }
}

pub struct ConvertExecutor;
impl ConvertExecutor {

    fn convert_none(
        path: PathBuf,
        bar: Progress,
        assets: &[PathBuf],
    ) -> GenericResult<()> {
        let toc_path = path.join(UTOCMETA);
        if std::fs::exists(toc_path.as_path())? {
            std::fs::remove_file(toc_path.as_path())?;
        }
        for asset in assets {
            let meta_path = path.join(asset).with_extension(UASSETMETA_EXTENSION);
            if std::fs::exists(meta_path.as_path())? {
                std::fs::remove_file(meta_path.as_path())?;
            }
            bar.set_message(asset.to_str().unwrap().to_owned());
            bar.set_position(bar.position() + 1);
        }
        Ok(())
    }

    fn convert_per_asset(
        path: PathBuf,
        bar: Progress,
        assets: &[PathBuf],
        version: EngineVersion
    ) -> GenericResult<()> {
        let mut metadata = UtocMetadata::default();
        let toc_path = path.join(UTOCMETA);
        metadata.add_from_utocmeta(
            std::fs::read(toc_path.as_path())?.as_ref(),
            version)?;
        std::fs::remove_file(toc_path.as_path())?;
        for asset in assets {
            let meta_path = path.join(asset).with_extension(UASSETMETA_EXTENSION);
            let package_id = convert_to_package_id(path.join(asset), path.as_path(), None);
            match metadata.get_manual_v2_import(package_id) {
                Some(v) => {
                    let mut writer = File::create(meta_path.as_path())?;
                    writer.ser(&v)?;
                },
                None => {
                    println!("{}: Could not get \"{}\" from utocmeta", console::style("Error:").red(), asset.to_str().unwrap());
                }
            }
            bar.set_message(asset.to_str().unwrap().to_owned());
            bar.set_position(bar.position() + 1);
        }
        Ok(())
    }

    fn convert_table(
        path: PathBuf,
        bar: Progress,
        assets: &[PathBuf],
        version: EngineVersion
    ) -> GenericResult<()> {
        let mut metadata = UtocMetadata::default();
        let toc_path = path.join(UTOCMETA);
        for asset in assets {
            let meta_path = path.join(asset).with_extension(UASSETMETA_EXTENSION);
            let package_id = convert_to_package_id(path.join(asset), path.as_path(), None);
            metadata.add_from_uassetmeta(package_id, meta_path.as_ref())?;
            std::fs::remove_file(meta_path.as_path())?;
            bar.set_message(asset.to_str().unwrap().to_owned());
            bar.set_position(bar.position() + 1);
        }
        let mut writer = BufWriter::new(File::create(toc_path)?);
        metadata.serialize(&mut writer, version.container_header_version())?;
        Ok(())
    }


    pub fn convert<P: AsRef<Path>>(
        input: P,
        fmt_from: AssetMetadata,
        fmt_to: AssetMetadata,
        assets: &[PathBuf],
        version: EngineVersion,
    ) -> GenericResult<()> {
        let path = input.as_ref().to_owned();
        let bar = Progress::new(assets.len() as u64)?;
        match fmt_to {
            AssetMetadata::None => Self::convert_none(path, bar, assets),
            AssetMetadata::PerAsset => Self::convert_per_asset(path, bar, assets, version),
            AssetMetadata::Table => Self::convert_table(path, bar, assets, version),
        }
    }
}