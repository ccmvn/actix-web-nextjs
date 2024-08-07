use std::{path::{Path, PathBuf}, sync::Arc};

use actix_files::NamedFile;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use anyhow::Result;
use glob::glob;
use once_cell::sync::Lazy;
use path_tree::PathTree;
use regex::{Captures, Regex};
use tracing::{error, trace};

use crate::error::SpaError;

/// Serve the SPA index file, falling back to the index file for unknown paths
pub async fn serve_index(
    req: ServiceRequest,
    index_file: &Arc<String>,
    static_resources_location: &Arc<String>,
    path_tree: &Arc<PathTree<String>>,
) -> Result<ServiceResponse, SpaError> {
    trace!("Serving default SPA page for path: {}", req.path());
    let (req, _) = req.into_parts();

    #[allow(unused_mut)]
    let mut file_path = match path_tree.find(req.path()) {
        Some((path, _)) => path.to_string(),
        None => req.path().to_string(),
    };

    #[cfg(feature = "wildcards")]
    {
        if file_path.contains('/') {
            let wildcard_path = convert_to_wildcard_path(&file_path)?;
            let pattern = format!("{}/{}.html", static_resources_location, wildcard_path);

            if let Some(matched_path) = glob(&pattern)?.find_map(Result::ok) {
                file_path = matched_path.to_str().unwrap().to_string();
            }
        }
    }

    let file_path = construct_file_path(&file_path, static_resources_location);

    let file = match NamedFile::open_async(&file_path).await {
        Ok(f) => f,
        Err(_) => {
            NamedFile::open_async(index_file.as_ref()).await.map_err(SpaError::from)?
        }
    };

    let res = file.into_response(&req);
    Ok(ServiceResponse::new(req, res))
}

/// Find and parse the build manifest file to construct a path tree
pub fn find_and_parse_build_manifest(static_resources_location: &str) -> Result<PathTree<String>, SpaError> {
    match find_build_manifest(static_resources_location)? {
        Some(build_manifest_path) => {
            let build_manifest_content = std::fs::read_to_string(&build_manifest_path)?;
            Ok(parse_build_manifest(build_manifest_content, static_resources_location))
        }
        None => Err(SpaError::BuildManifestNotFound)
    }
}

/// Find the build manifest file based on a glob pattern
fn find_build_manifest(static_resources_location: &str) -> Result<Option<PathBuf>, SpaError> {
    let pattern = format!("{}/_next/**/_buildManifest.js", static_resources_location);
    match glob(&pattern) {
        Ok(paths) => Ok(paths.filter_map(Result::ok).next()),
        Err(err) => {
            error!("Failed to read glob pattern: {}: {:?}", pattern, err);
            Err(SpaError::GlobPatternError(err))
        }
    }
}

/// Parse the build manifest content to create a path tree
fn parse_build_manifest(build_manifest: String, static_resources_location: &str) -> PathTree<String> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""([^,]+)":\s*\["[^,]+"]"#).unwrap());

    let mut tree = PathTree::new();
    let resources_path = Path::new(static_resources_location);

    for (_, [path]) in RE.captures_iter(&build_manifest).map(|c| c.extract()) {
        let value = resources_path
            .join(format!(
                "{}.html",
                if path == "/" {
                    "index"
                } else {
                    path.strip_prefix("/").unwrap()
                }
            ))
            .to_str()
            .unwrap()
            .to_string();

        let path = convert_dynamic_path(path).replace(".html", "");
        let _ = tree.insert(&path, value);
    }

    trace!("Build manifest parsed successfully");
    tree
}

/// Convert dynamic paths with parameters to a consistent format
fn convert_dynamic_path(path: &str) -> String {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?<param>\[[^]]+])"#).unwrap());

    RE.replace_all(path, |caps: &Captures| {
        format!(":{}", &caps["param"].replace("[", "").replace("]", ""))
    })
        .to_string()
}

/// Convert file paths with dynamic segments to wildcard format
#[cfg(feature = "wildcards")]
fn convert_to_wildcard_path(file_path: &str) -> Result<String, SpaError> {
    Ok(file_path
        .split('/')
        .map(|segment| if segment.parse::<u32>().is_ok() { "*" } else { segment })
        .collect::<Vec<&str>>()
        .join("/"))
}

/// Construct the final file path for the requested resource
pub fn construct_file_path(file_path: &str, static_resources_location: &Arc<String>) -> PathBuf {
    let constructed_path = if !file_path.contains('*') && !file_path.ends_with(".html") {
        format!(
            "{}/{}.html",
            static_resources_location.trim_end_matches('/'),
            file_path.trim_start_matches('/').trim_end_matches('/')
        )
    } else {
        file_path.to_string()
    };

    PathBuf::from(constructed_path)
}
