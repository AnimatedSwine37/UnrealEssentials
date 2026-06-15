use std::fs::File;
use std::io::{BufReader, Cursor};
use std::ops::Deref;
use std::path::PathBuf;
use clap::{Parser, ValueEnum};
use retoc::{AesKey, Config, EIoChunkType, FGuid, Toc};
use retoc::version::EngineVersion;
use crate::GenericResult;
use std::str::FromStr;
use std::sync::Arc;
use anyhow::{anyhow, Context};
use console::{Style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use retoc::container_header::{EIoContainerHeaderVersion, FIoContainerHeader};
use retoc::file_pool::FilePool;
use retoc::ser::{ReadExt, WriteExt, Writeable};
use retoc::zen::FZenPackageSummary;
use utoc_lib::metadata::UtocMetadata;
use utoc_lib::store::{get_asset_exports_new, get_asset_exports_old};

#[derive(ValueEnum, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
enum AssetMetadata {
    None,
    Table,
    PerAsset
}

#[derive(Parser, Debug)]
struct Args {
    // #[arg(short, long)]
    #[arg(help = "The file path to the .utoc to extract")]
    input: String,
    #[arg(long)]
    aes_key: Option<String>,
    #[arg(short, long, help = "Define a set of paths in the archive to extract. If not specified, everything will be extracted")]
    include: Vec<String>,
    #[arg(short, long)]
    metadata: Option<AssetMetadata>,
    #[arg(long)]
    override_version: Option<EngineVersion>,
    #[arg(long, help = "Set the name of the root folder. By default, this is \"Game\"")]
    root_name: Option<String>,
    #[arg(short, long)]
    #[arg(help = "The folder to extract into. By default, this will be a in a folder adjacent to the .utoc")]
    output: Option<String>
}

fn create_config(args: &Args) -> GenericResult<Arc<Config>> {
    let mut config = Config {
        container_header_version_override: args.override_version.map(|v| v.container_header_version()),
        toc_version_override: args.override_version.map(|v| v.toc_version()),
        ..Default::default()
    };
    if let Some(aes) = args.aes_key.clone() {
        config.aes_keys.insert(FGuid::default(), AesKey::from_str(&aes)?);
    }
    Ok(Arc::new(config))
}

#[derive(Debug)]
struct Progress(ProgressBar);

impl Progress {
    pub fn new(count: u64) -> GenericResult<Self> {
        let bar = ProgressBar::new(count);
        let color_fmt = match Term::stdout().features().true_colors_supported() {
            true => "#DA70D6/#9932CC", false => "135/90"
        };
        let template_fmt = format!("[{{elapsed_precise}}] {{bar:40.{}}} {{pos:>7}}/{{len:7}} ({{percent_precise}}%) {{msg}}", color_fmt);
        let bar_style = ProgressStyle::with_template(&template_fmt)?
            .progress_chars("##-");
        bar.set_style(bar_style);
        bar.tick();
        Ok(Self(bar))
    }
}

impl Deref for Progress {
    type Target = ProgressBar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) fn execute() -> GenericResult<()> {
    let args = Args::parse();
    let config = create_config(&args)?;
    let metadata = args.metadata.unwrap_or(AssetMetadata::PerAsset);
    let toc: Toc = BufReader::new(File::open(&args.input)?).de_ctx(config.clone())?;
    let cas_path = PathBuf::from(&args.input).with_extension("ucas");
    let cas = FilePool::new(&cas_path, 1)?;
    let header = if let Some((id, offset)) = toc.chunk_id_map.iter().find(
        |(id, _)| id.get_chunk_type() == EIoChunkType::ContainerHeader) {
        let mut file_lock = cas.acquire()?;
        let data = toc.read(&mut file_lock.file(), *offset)
            .with_context(|| format!("Failed to read chunk {id:?}"))?;
        FIoContainerHeader::deserialize(&mut Cursor::new(&data), config.container_header_version_override)
    } else { Err(anyhow!("Could not find the container header in \"{}\"", cas_path.to_str().unwrap())) }?;
    // No metadata warning/error
    let warning = Style::new().yellow();
    if metadata == AssetMetadata::None {
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

    let input = PathBuf::from(&args.input);
    let output_default = input.parent().unwrap().join(
        input.file_stem().map_or("Archive", |v| v.to_str().unwrap()));
    let output = args.output.map_or(output_default, |v| PathBuf::from(v));
    let root_folder = args.root_name.as_ref().map_or("Game", |v| v.as_str());
    let mount_point = toc.directory_index.mount_point.to_string();
    let content = if mount_point.len() > utoc_lib::assets::MOUNT_POINT.len() {
        output.join(&mount_point[utoc_lib::assets::MOUNT_POINT.len()..])
    } else {
        output.join(root_folder).join("Content")
    };
    let mut cas = BufReader::new(File::open(&cas_path)?);

    println!("Metadata type: {:?}", metadata);
    println!("Writing into {}", output.to_str().unwrap());

    let assets: Vec<_> = toc.chunk_id_map.iter().filter_map(|(id, offset)|
        toc.file_map_rev.get(offset).map(|f| (id, f.clone(), *offset))).collect();

    let bar = Progress::new(assets.len() as u64)?;
    let mut toc_meta = UtocMetadata::default();

    for (id, path, offset) in &assets {
        let store_entry = header.get_store_entry(id.get_package_id());
        let asset_path = content.join(path);
        let data = toc.read(&mut cas, *offset as _)?;
        let dir_path = asset_path.parent().unwrap();
        std::fs::create_dir_all(dir_path)?;
        std::fs::write(&asset_path, &data)?;
        if let Some(store_entry) = store_entry {
            match metadata {
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
        bar.set_message(path.clone());
        bar.set_position(bar.position() + 1);
    }
    if metadata == AssetMetadata::Table {
        let mut meta_file = File::create(output.join(".utocmeta"))?;
        toc_meta.serialize(&mut meta_file, header.version)?;
    }
    println!("Wrote {} files", bar.position());
    Ok(())
}