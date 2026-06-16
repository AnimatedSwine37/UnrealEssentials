use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use anyhow::{anyhow, Context};
use chrono::Utc;
use eframe::Frame;
use egui::{Align, Button, Color32, ComboBox, Id, Layout, RichText, ScrollArea, TextEdit,  Ui, Widget, WidgetText};
use egui_dock::{DockArea, DockState, Style, TabViewer};
use retoc::{AesKey, Config, EIoChunkType, FGuid, Toc};
use retoc::container_header::{EIoContainerHeaderVersion, FIoContainerHeader};
use retoc::file_pool::FilePool;
use retoc::ser::{ReadExt, WriteExt};
use retoc::version::EngineVersion;
use rfd::FileDialog;
use utoc_lib::metadata::UtocMetadata;
use crate::common::AssetMetadata;
use crate::GenericResult;

pub trait AppAction {
    fn ui(&mut self, ui: &mut Ui);
}

pub struct App {
    tabs: DockState<AppTab>
}

pub struct AppTab {
    title: String,
    contents: Option<Box<dyn AppAction>>
}

#[derive(Debug)]
pub struct FilePicker {
    label: &'static str,
    path: String,
    open_dialog: fn() -> Option<PathBuf>
}

impl FilePicker {
    fn ui(&mut self, ui: &mut Ui) -> bool {
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

            if ui.button("Select").clicked()
                && let Some(path) = (self.open_dialog)() {
                self.path = path.to_str().unwrap().to_string();
                loaded = true;
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
    pub fn new(label: &'static str, open_dialog: fn() -> Option<PathBuf>) -> Self {
        Self { label, path: String::new(), open_dialog }
    }

    pub fn get_path(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }
}

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

pub struct UnpackInfo {
    text: String,
    color: Color32
}

impl UnpackInfo {
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
    info: Option<UnpackInfo>,
}

impl Default for UnpackAction {
    fn default() -> Self {
        Self {
            input: FilePicker::new("UTOC to unpack: ", || {
                FileDialog::new()
                    .add_filter("IO Store TOC", &["utoc"])
                    .pick_file()
            }),
            aes_key: String::new(),
            include: HashSet::new(),
            metadata: AssetMetadata::PerAsset,
            can_override_version: false,
            override_version: EngineVersion::UE5_3,
            root_name: "Game".to_string(),
            output: FilePicker::new("Output folder: ", || {
                FileDialog::new()
                    .pick_folder()
            }),
            toc: None,
            toc_root: None,
            info: None,
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
        let start = Instant::now();
        let config = self.create_config()?;
        self.toc = Some(BufReader::new(File::open(self.input.get_path())?).de_ctx(config.clone())?);
        let toc = self.toc.as_ref().unwrap();
        let mount_point = toc.directory_index.mount_point.to_string();
        let root_parts: Vec<_> = if mount_point.len() > utoc_lib::assets::MOUNT_POINT.len() {
            (&mount_point[utoc_lib::assets::MOUNT_POINT.len()..]).split("/").filter(|v| !v.is_empty()).collect()
        } else {
            vec![self.root_name.as_ref(), "Content"]
        };
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

        let mount_point = root_parts.join("/");
        toc.directory_index.iter_root(|_, path| {
            let mut current = self.toc_root.as_mut().unwrap();
            for part in root_parts[1..].iter() {
                match current {
                    TocEntry::Directory(d) => {
                        current = d.children.iter_mut()
                            .find(|v| v.get_name() == *part).unwrap();
                    },
                    _ => panic!("Expected a folder")
                }
            }
            for (i, name) in path[..path.len() - 1].iter().enumerate() {
                match current {
                    TocEntry::Directory(d) => {
                        let full_path = format!("{}/{}", &mount_point, path[..i + 1].join("/"));
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
                    let full_path = format!("{}/{}", &mount_point, path.join("/"));
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
        let mut output = self.output.get_path();
        let mount_point = toc.directory_index.mount_point.to_string();
        let content = if mount_point.len() > utoc_lib::assets::MOUNT_POINT.len() {
            output.join(&mount_point[utoc_lib::assets::MOUNT_POINT.len()..])
        } else {
            output.join(&self.root_name).join("Content")
        };
        let content_str = if mount_point.len() > utoc_lib::assets::MOUNT_POINT.len() {
            (&mount_point[utoc_lib::assets::MOUNT_POINT.len()..]).to_string()
        } else {
            format!("{}/Content/", self.root_name)
        };
        let mut cas = BufReader::new(File::open(&cas_path)?);

        println!("Metadata type: {:?}", self.metadata);
        println!("Writing into {}", output.to_str().unwrap());

        let assets: Vec<_> = toc.chunk_id_map.iter().filter_map(|(id, offset)|
            toc.file_map_rev.get(offset).map(|f| (id, f.clone(), *offset))).collect();

        let mut toc_meta = UtocMetadata::default();
        for (id, path, offset) in &assets {
            if !self.include.contains(&format!("{}{}", &content_str, path)) {
                continue;
            }
            let store_entry = header.get_store_entry(id.get_package_id());
            let asset_path = content.join(path);
            let data = toc.read(&mut cas, *offset as _)?;
            let dir_path = asset_path.parent().unwrap();
            std::fs::create_dir_all(dir_path)?;
            std::fs::write(&asset_path, &data)?;
            if let Some(store_entry) = store_entry {
                match self.metadata {
                    AssetMetadata::PerAsset => {
                        let meta_path = asset_path.with_extension("uassetmeta");
                        let mut meta_file = File::create(meta_path)?;
                        meta_file.ser(&store_entry)?;
                    },
                    AssetMetadata::Table => {
                        toc_meta.add_from_store_entry(id.get_package_id(), store_entry)?;
                    },
                    _ => {}
                }
            }
        }
        if self.metadata == AssetMetadata::Table {
            let mut meta_file = File::create(output.join(".utocmeta"))?;
            toc_meta.serialize(&mut meta_file, header.version)?;
        }
        Ok(())
    }
}

impl AppAction for UnpackAction {
    fn ui(&mut self, ui: &mut Ui) {
        if let Some(err) = &self.info {
            ui.label(RichText::new(&err.text)
                .color(err.color));
        }
        if self.input.ui(ui) {
            self.info = match self.load_utoc() {
                Ok(_) => {
                    if self.output.path.is_empty() {
                        self.output.path = self.input.get_path().parent().unwrap()
                            .to_str().unwrap().to_string();
                    }
                    None
                },
                Err(e) => Some(UnpackInfo::error(format!("Failed to load UTOC: {}", e.to_string())))
            }
        }
        self.output.ui(ui);
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("AES Key: ");
            ui.add(TextEdit::singleline(&mut self.aes_key)
                .desired_width(ui.max_rect().width())
                .char_limit(66));
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
                ComboBox::new("engine_version", "")
                    .selected_text(format!("{:?}", self.override_version))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.override_version, EngineVersion::UE4_25, format!("{:?}", EngineVersion::UE4_25));
                        ui.selectable_value(&mut self.override_version, EngineVersion::UE4_26, format!("{:?}", EngineVersion::UE4_26));
                        ui.selectable_value(&mut self.override_version, EngineVersion::UE4_27, format!("{:?}", EngineVersion::UE4_27));
                        ui.selectable_value(&mut self.override_version, EngineVersion::UE5_0, format!("{:?}", EngineVersion::UE5_0));
                        ui.selectable_value(&mut self.override_version, EngineVersion::UE5_1, format!("{:?}", EngineVersion::UE5_1));
                        ui.selectable_value(&mut self.override_version, EngineVersion::UE5_2, format!("{:?}", EngineVersion::UE5_2));
                        ui.selectable_value(&mut self.override_version, EngineVersion::UE5_3, format!("{:?}", EngineVersion::UE5_3));
                        ui.selectable_value(&mut self.override_version, EngineVersion::UE5_4, format!("{:?}", EngineVersion::UE5_4));
                        ui.selectable_value(&mut self.override_version, EngineVersion::UE5_5, format!("{:?}", EngineVersion::UE5_5));
                        ui.selectable_value(&mut self.override_version, EngineVersion::UE5_6, format!("{:?}", EngineVersion::UE5_6));
                        ui.selectable_value(&mut self.override_version, EngineVersion::UE5_7, format!("{:?}", EngineVersion::UE5_7));
                    });
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
                            Some(UnpackInfo::info(fmt))
                        },
                        Err(e) => Some(UnpackInfo::error(format!("Failed to unpack assets: {}", e.to_string())))
                    };
                }
            });
        });
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
                    title: "Merge".to_string(),
                    contents: None
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
    eframe::run_native(
        "Unreal Essentials UTOC Extractor",
        options,
        Box::new(|cc| {
            Ok(Box::<App>::default())
        })
    )?;
    Ok(())
}