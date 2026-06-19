use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};
use std::path::{Component, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::{anyhow, Context};
use chrono::Utc;
use eframe::emath::Align;
use eframe::epaint::Color32;
use egui::{Button, ComboBox, Layout, RichText, ScrollArea, TextEdit, Ui};
use ini::Ini;
use retoc::{AesKey, Config, EIoChunkType, FGuid, Toc};
use retoc::container_header::{EIoContainerHeaderVersion, FIoContainerHeader};
use retoc::file_pool::FilePool;
use retoc::ser::{ReadExt, WriteExt};
use retoc::version::EngineVersion;
#[cfg(target_os = "windows")]
use wfd::{ DialogParams, FOS_PICKFOLDERS };
#[cfg(not(target_os = "windows"))]
use rfd::FileDialog;
use utoc_lib::metadata::UtocMetadata;
use crate::cli::Progress;
use crate::common::{get_root_path, get_default_directory, ActionInfo, AssetMetadata, FilePicker, UIComponent, set_default_directory};
use crate::GenericResult;
use crate::gui::AppAction;

pub struct TocFile {
    name: String,
    path: String,
    enabled: bool,
}

impl TocFile {
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, value: bool, ctx: &mut HashSet<String>) {
        self.enabled = value;
        match value {
            true => ctx.insert(self.path.clone()),
            false => ctx.remove(&self.path)
        };
    }
}

pub struct TocDirectory {
    name: String,
    path: String,
    enabled: bool,
    children: Vec<TocEntry>
}

impl TocDirectory {
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, value: bool, ctx: &mut HashSet<String>) {
        self.enabled = value;
        for child in &mut self.children {
            child.set_enabled(value, ctx);
        }
    }
}

pub enum TocEntry {
    File(TocFile),
    Directory(TocDirectory)
}

impl TocEntry {
    pub fn file(name: String, path: String, ctx: &mut HashSet<String>) -> TocEntry {
        ctx.insert(path.clone());
        Self::File(TocFile {
            name,
            path,
            enabled: true
        })
    }

    pub fn directory(name: String, path: String) -> TocEntry {
        Self::Directory(TocDirectory {
            name,
            path,
            enabled: true,
            children: vec![]
        })
    }

    pub fn get_name(&self) -> &str {
        match self {
            Self::File(f) => f.get_name(),
            Self::Directory(d) => d.get_name()
        }
    }

    pub fn get_enabled(&self) -> bool {
        match self {
            Self::File(f) => f.get_enabled(),
            Self::Directory(d) => d.get_enabled()
        }
    }

    pub fn set_enabled(&mut self, value: bool, ctx: &mut HashSet<String>) {
        match self {
            Self::File(f) => f.set_enabled(value, ctx),
            Self::Directory(d) => d.set_enabled(value, ctx)
        }
    }
}

pub struct Debounce {
    last_trigger: Option<Instant>,
    duration: Duration,
}

impl Debounce {
    pub fn new(ms: u64) -> Self {
        Self {
            last_trigger: None,
            duration: Duration::from_millis(ms),
        }
    }

    pub fn fire(&mut self) {
        self.last_trigger = Some(Instant::now());
    }

    pub fn update(&mut self) -> bool {
        if let Some(last) = &self.last_trigger {
            let now = Instant::now();
            if now.duration_since(*last) > self.duration {
                self.last_trigger = None;
                return true;
            }
        }
        false
    }
}

// #[derive(Debug)]
pub struct UnpackAction {
    input: FilePicker,
    aes_key: String,
    include: HashSet<String>,
    metadata: AssetMetadata,
    can_override_version: bool,
    override_version: EngineVersion,
    root_name: String,
    output: FilePicker,
    toc: Option<Toc>,
    toc_root: Option<TocEntry>,
    info: Option<ActionInfo>,
    aes_key_reload: Debounce,
}

const UNPACK_INPUT_TITLE: &'static str = "Select UTOC to unpack";
const UNPACK_INPUT_KEY: &'static str = "unpack_input";

const UNPACK_OUTPUT_TITLE: &'static str = "Select folder to output Zen assets to";
const UNPACK_OUTPUT_KEY: &'static str = "unpack_output";

impl UnpackAction {
    #[cfg(not(target_os = "windows"))]
    fn unpack_input(mut ini: Option<Ini>) -> Option<PathBuf> {
        let dialog = FileDialog::new()
            .add_filter("IO Store TOC", &["utoc"])
            .set_title(UNPACK_INPUT_TITLE);
        let path = get_default_directory(dialog, ini.as_mut(), UNPACK_INPUT_KEY)
            .pick_file();
        if path.is_some() {
            set_default_directory(path.as_ref(), ini.as_mut(), UNPACK_INPUT_KEY);
        }
        path
    }

    #[cfg(target_os = "windows")]
    fn unpack_input(mut ini: Option<Ini>) -> Option<PathBuf> {
        let mut params = DialogParams {
            title: UNPACK_INPUT_TITLE,
            file_types: vec![("IO Store TOC", "*.utoc")],
            default_extension: "utoc",
            ..Default::default()
        };
        if let Some(folder) = get_default_directory(ini.as_mut(), UNPACK_INPUT_KEY) {
            params.folder = folder;
        }
        let result = wfd::open_dialog(params).map(|v| v.selected_file_path).ok();
        if let Some(result) = &result {
            set_default_directory(Some(result.parent().unwrap()), ini.as_mut(), UNPACK_INPUT_KEY);
        }
        result
    }

    #[cfg(not(target_os = "windows"))]
    fn unpack_output(mut ini: Option<Ini>) -> Option<PathBuf> {
        let dialog = FileDialog::new()
            .set_title(UNPACK_OUTPUT_TITLE);
        let path = get_default_directory(dialog, ini.as_mut(), UNPACK_OUTPUT_KEY)
            .pick_folder();
        if path.is_some() {
            set_default_directory(path.as_ref(), ini.as_mut(), UNPACK_OUTPUT_KEY);
        }
        path
    }

    #[cfg(target_os = "windows")]
    fn unpack_output(mut ini: Option<Ini>) -> Option<PathBuf> {
        let mut params = DialogParams {
            title: UNPACK_OUTPUT_TITLE,
            options: FOS_PICKFOLDERS,
            ..Default::default()
        };
        if let Some(folder) = get_default_directory(ini.as_mut(), UNPACK_OUTPUT_KEY) {
            params.folder = folder;
        }
        let result = wfd::open_dialog(params).map(|v| v.selected_file_path).ok();
        if let Some(result) = &result {
            set_default_directory(Some(result.as_path()), ini.as_mut(), UNPACK_OUTPUT_KEY);
        }
        result
    }
}

impl Default for UnpackAction {
    fn default() -> Self {
        Self {
            input: FilePicker::new("UTOC to unpack: ", Self::unpack_input),
            aes_key: String::new(),
            include: HashSet::new(),
            metadata: AssetMetadata::PerAsset,
            can_override_version: false,
            override_version: EngineVersion::UE5_3,
            root_name: "Game".to_string(),
            output: FilePicker::new("Output folder: ", Self::unpack_output),
            toc: None,
            toc_root: None,
            info: None,
            aes_key_reload: Debounce::new(250)
        }
    }
}

impl UnpackAction {
    fn get_override_version(&self) -> Option<EngineVersion> {
        match self.can_override_version {
            true => Some(self.override_version),
            false => None
        }
    }

    fn get_aes_key(&self) -> Option<&str> {
        match self.aes_key.is_empty() {
            false => Some(self.aes_key.as_str()),
            true => None
        }
    }

    fn create_config(&self) -> GenericResult<Arc<Config>> {
        let mut config = Config {
            container_header_version_override: self.get_override_version().map(|v| v.container_header_version()),
            toc_version_override: self.get_override_version().map(|v| v.toc_version()),
            ..Default::default()
        };
        if let Some(aes) = self.get_aes_key() {
            config.aes_keys.insert(FGuid::default(), AesKey::from_str(aes)?);
        }
        Ok(Arc::new(config))
    }

    fn load_utoc(&mut self) -> GenericResult<()> {
        self.include.clear();
        let start = Instant::now();
        let config = self.create_config()?;
        self.toc = Some(BufReader::new(File::open(self.input.get_path())?).de_ctx(config.clone())?);
        let toc = self.toc.as_ref().unwrap();
        let mount_point = toc.directory_index.mount_point.to_string();
        let root_parts: Option<Vec<_>> = if mount_point.len() > utoc_lib::assets::MOUNT_POINT.len() {
            Some((&mount_point[utoc_lib::assets::MOUNT_POINT.len()..])
                .split("/").filter(|v| !v.is_empty()).collect())
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
                true => Some(vec![self.root_name.as_ref(), "Content"]),
                false => None
                // false => (vec!["Root"], true),
            }
        };
        match &root_parts {
            Some(root_parts) => {
                self.toc_root = Some(TocEntry::directory(root_parts[0].to_string(), root_parts[0].to_string()));
                let mut root = self.toc_root.as_mut().unwrap();
                for (i, part) in root_parts[1..].iter().enumerate() {
                    match root {
                        TocEntry::Directory(d) => {
                            d.children.push(TocEntry::directory(
                                part.to_string(),
                                root_parts[..i + 1].join("/")));
                            root = d.children.last_mut().unwrap();
                        },
                        _ => panic!("Expected a folder")
                    }
                }
            },
            None => {
                let name = "Root".to_string();
                self.toc_root = Some(TocEntry::directory(name.clone(), name));
            }
        }
        let mount_point = root_parts.as_ref().map(|v| v.join("/"));
        toc.directory_index.iter_root(|_, path| {
            let mut current = self.toc_root.as_mut().unwrap();
            if let Some(root_parts) = &root_parts {
                for part in root_parts[1..].iter() {
                    match current {
                        TocEntry::Directory(d) => {
                            current = d.children.iter_mut()
                                .find(|v| v.get_name() == *part).unwrap();
                        },
                        _ => panic!("Expected a folder")
                    }
                }
            }
            for (i, name) in path[..path.len() - 1].iter().enumerate() {
                match current {
                    TocEntry::Directory(d) => {
                        let full_path = match &mount_point {
                            Some(mount) => format!("{}/{}", mount, path[..i + 1].join("/")),
                            None => path[..i + 1].join("/").to_string()
                        };
                        if d.children.iter_mut()
                            .find(|v| v.get_name() == *name).is_none() {
                            d.children.push(TocEntry::directory(name.to_string(), full_path));
                        }
                        current = d.children.iter_mut().find(|v| v.get_name() == *name).unwrap();
                    },
                    _ => panic!("Expected a folder")
                }
            }
            match current {
                TocEntry::Directory(d) => {
                    let full_path = match &mount_point {
                        Some(mount) => format!("{}/{}", mount, path.join("/")),
                        None => path.join("/")
                    };
                    d.children.push(TocEntry::file(path.last().unwrap().to_string(), full_path, &mut self.include));
                },
                _ => panic!("Expected a folder")
            }
        });
        let time = Instant::now().duration_since(start).as_micros() as f64 / 1000.;
        println!("Loaded {} assets in {} ms", self.toc.as_ref().unwrap().file_map.len(), time);
        Ok(())
    }

    fn draw_toc_tree(node: &mut TocEntry, ctx: &mut HashSet<String>, ui: &mut Ui) {
        match node {
            TocEntry::File(f) => {
                ui.horizontal(|ui| {
                    ui.label(&f.name);
                    if ui.checkbox(&mut f.enabled, "").clicked() {
                        f.set_enabled(f.enabled, ctx);
                    }
                });
            },
            TocEntry::Directory(d) => {
                let id = ui.make_persistent_id(&d.path);
                egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
                    .show_header(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(&d.name);
                            if ui.checkbox(&mut d.enabled, "").clicked() {
                                d.set_enabled(d.enabled, ctx);
                            }
                        });
                    })
                    .body(|ui| {
                        for child in &mut d.children {
                            Self::draw_toc_tree(child, ctx, ui);
                        }
                    });
            }
        }
    }

    fn unpack(&self) -> GenericResult<()> {
        let cas_path = self.input.get_path().with_extension("ucas");
        let cas = FilePool::new(&cas_path, 1)?;
        let toc = self.toc.as_ref().unwrap();
        let header = if let Some((id, offset)) = toc.chunk_id_map.iter().find(
            |(id, _)| id.get_chunk_type() == EIoChunkType::ContainerHeader) {
            let mut file_lock = cas.acquire()?;
            let data = toc.read(&mut file_lock.file(), *offset)
                .with_context(|| format!("Failed to read chunk {id:?}"))?;
            FIoContainerHeader::deserialize(
                &mut Cursor::new(&data),
                self.get_override_version().map(|v| v.container_header_version()))
        } else { Err(anyhow!("Could not find the container header in \"{}\"", cas_path.to_str().unwrap())) }?;
        // No metadata warning/error
        let warning = console::Style::new().yellow();
        if self.metadata == AssetMetadata::None {
            match header.version {
                EIoContainerHeaderVersion::Initial => {
                    println!("{}: It's recommended to generate asset metadata to prevent issues trying to determine asset dependencies.", warning.apply_to("WARNING"));
                },
                v if v < EIoContainerHeaderVersion::NoExportInfo => {
                    return Err(anyhow!("Metadata is required").into_boxed_dyn_error());
                },
                _ => {}
            }
        }
        let output = self.output.get_path();
        let mount_point = toc.directory_index.mount_point.to_string();
        let content = get_root_path(output.as_path(), &mount_point, &toc, &self.root_name);
        let mut cas = BufReader::new(File::open(&cas_path)?);

        println!("Metadata type: {:?}", self.metadata);
        println!("Writing into {}", output.to_str().unwrap());

        let assets: Vec<_> = toc.chunk_id_map.iter().filter_map(|(id, offset)|
            toc.file_map_rev.get(offset).map(|f| (id, f.clone(), *offset))).collect();

        let bar = Progress::new(assets.len() as u64)?;

        let mut toc_meta = UtocMetadata::default();
        for (id, path, offset) in &assets {
            let os_path = content.join(path);
            let asset_path = os_path.strip_prefix(output.as_path())?
                .components().filter_map(|c| match c {
                Component::Normal(c) => Some(c.to_str().unwrap().to_string()),
                _ => None }).collect::<Vec<String>>().join("/");
            if !self.include.contains(&asset_path) {
                continue;
            }
            let store_entry = header.get_store_entry(id.get_package_id());
            let data = toc.read(&mut cas, *offset as _)?;
            std::fs::create_dir_all(os_path.parent().unwrap())?;
            std::fs::write(&os_path, &data)?;
            if let Some(store_entry) = store_entry {
                match self.metadata {
                    AssetMetadata::PerAsset => {
                        let meta_path = os_path.with_extension("uassetmeta");
                        let mut meta_file = File::create(meta_path)?;
                        meta_file.ser(&store_entry)?;
                    },
                    AssetMetadata::Table => {
                        toc_meta.add_from_store_entry(id.get_package_id(), store_entry)?;
                    },
                    _ => {}
                }
            }

            bar.set_message(path.clone());
            bar.set_position(bar.position() + 1);
        }
        if self.metadata == AssetMetadata::Table {
            let mut meta_file = File::create(output.join(".utocmeta"))?;
            toc_meta.serialize(&mut meta_file, header.version)?;
        }
        Ok(())
    }

    fn load_utoc_gui(&mut self) {
        self.info = match self.load_utoc() {
            Ok(_) => {
                if self.output.path.is_empty() {
                    let in_path = self.input.get_path();
                    let name = in_path.file_stem().unwrap().to_str().unwrap();
                    self.output.path = in_path.parent().unwrap()
                        .join(name).to_str().unwrap().to_string();
                }
                None
            },
            Err(e) => Some(ActionInfo::error(format!("Failed to load UTOC: {}", e.to_string())))
        }
    }
}

impl AppAction for UnpackAction {
    fn ui(&mut self, ui: &mut Ui) {
        if let Some(err) = &self.info {
            ui.label(RichText::new(&err.text)
                .color(err.color));
        }
        if self.input.ui(ui) || self.aes_key_reload.update() {
            self.load_utoc_gui();
        }
        self.output.ui(ui);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("AES Key: ");
            if ui.add(TextEdit::singleline(&mut self.aes_key)
                .desired_width(ui.max_rect().width())
                .char_limit(66)).changed() &&
                self.aes_key.len() == 66 &&
                self.aes_key.starts_with("0x") {
                    self.aes_key_reload.fire();
            }
        });
        if self.aes_key.len() > 0 && (!self.aes_key.starts_with("0x") || self.aes_key.len() != 66) {
            ui.label(RichText::new(
                "AES Key must be a 64 digit long hexadecimal number starting with \"0x\"")
                .color(Color32::from_rgb(255, 0, 0)));
        }
        ui.horizontal(|ui| {
            ui.label("Metadata Format: ");
            ComboBox::new("metadata_format", "")
                .selected_text(format!("{}", self.metadata))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.metadata, AssetMetadata::None, format!("{}", AssetMetadata::None));
                    ui.selectable_value(&mut self.metadata, AssetMetadata::PerAsset, format!("{}", AssetMetadata::PerAsset));
                    ui.selectable_value(&mut self.metadata, AssetMetadata::Table, format!("{}", AssetMetadata::Table));
                });
            ui.label("Override Engine Version: ");
            ui.checkbox(&mut self.can_override_version, "");
            ui.add_enabled_ui(self.can_override_version, |ui| {
                UIComponent::engine_version_combobox(ui, &mut self.override_version);
            });
        });
        if let Some(toc) = self.toc.as_ref() {
            ui.label(format!("{} assets ({} selected)", toc.file_map.len(), self.include.len()));
            ui.separator();
        }
        let remain = ui.max_rect().height() - ui.min_rect().height() - 48.;
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_width(ui.available_width())
            .max_height(remain)
            .show(ui, |ui| {
                if let Some(root) = self.toc_root.as_mut() {
                    Self::draw_toc_tree(root, &mut self.include, ui);
                }
            });
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            ui.add_enabled_ui(self.toc.is_some(), |ui| {
                if ui.add(Button::new("Unpack")
                    .min_size([ui.max_rect().width(), 32.].into()))
                    .clicked() {
                    let start = Instant::now();
                    self.info = match self.unpack() {
                        Ok(_) => {
                            let duration = Instant::now().duration_since(start)
                                .as_micros() as f64 / 1000.;
                            let time = Utc::now().time().format("%-I:%M:%S");
                            let fmt = format!(
                                "Unpacked {} assets! (Took {} ms at {})", self.include.len(), duration, time);
                            Some(ActionInfo::info(fmt))
                        },
                        Err(e) => Some(ActionInfo::error(format!("Failed to unpack assets: {}", e.to_string())))
                    };
                }
            });
        });
    }
}