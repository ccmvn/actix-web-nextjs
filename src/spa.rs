use std::borrow::Cow;
use std::sync::Arc;

use actix_files::Files;
use actix_web::dev::HttpServiceFactory;
use path_tree::PathTree;
use tracing::warn;

use crate::spa_service::SpaService;
use crate::utils::{find_and_parse_build_manifest, serve_index};

/// Single Page App (SPA) service builder
///
/// # Examples
/// ```
/// # use actix_web::App;
/// # use actix_web_nextjs::spa;
///
/// let app = App::new()
///     // API routes and other services
///     .service(
///         spa()
///             .index_file("dist/index.html")
///             .static_resources_mount("dist")
///             .static_resources_location("/")
///             .finish()
///     );
/// ```
#[derive(Debug, Clone)]
pub struct Spa {
    index_file: Cow<'static, str>,
    static_resources_mount: Cow<'static, str>,
    static_resources_location: Cow<'static, str>,
}

impl Spa {
    /// Create a new `Spa` instance with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the index file for the SPA
    pub fn index_file(mut self, index_file: impl Into<String>) -> Self {
        self.index_file = Cow::Owned(index_file.into());
        self
    }

    /// Set the mount point for static resources
    pub fn static_resources_mount(mut self, static_resources_mount: impl Into<String>) -> Self {
        self.static_resources_mount = Cow::Owned(static_resources_mount.into());
        self
    }

    /// Set the location for static resources
    pub fn static_resources_location(mut self, static_resources_location: impl Into<String>) -> Self {
        self.static_resources_location = Cow::Owned(static_resources_location.into());
        self
    }

    /// Finalize the configuration and return the SPA service
    pub fn finish(self) -> impl HttpServiceFactory {
        let index_file = Arc::new(self.index_file.into_owned());
        let static_resources_location = Arc::new(self.static_resources_location.into_owned());
        let static_resources_mount = self.static_resources_mount.into_owned();

        let path_tree = match find_and_parse_build_manifest(&static_resources_location) {
            Ok(tree) => Arc::new(tree),
            Err(e) => {
                warn!("Failed to parse build manifest: {}. Using default path tree.", e);
                Arc::new(PathTree::default())
            }
        };

        let files = Files::new(&static_resources_mount, static_resources_location.as_str())
            .index_file("extremely-unlikely-to-exist-!@$%^&*.txt")
            .default_handler({
                let index_file = Arc::clone(&index_file);
                let static_resources_location = Arc::clone(&static_resources_location);
                let path_tree = Arc::clone(&path_tree);

                move |req| {
                    let index_file = Arc::clone(&index_file);
                    let static_resources_location = Arc::clone(&static_resources_location);
                    let path_tree = Arc::clone(&path_tree);

                    async move {
                        serve_index(req, &index_file, &static_resources_location, &path_tree)
                            .await
                            .map_err(actix_web::Error::from)
                    }
                }
            });

        SpaService {
            index_file,
            static_resources_location,
            files,
            path_tree,
        }
    }
}

/// Default implementation for `Spa`
impl Default for Spa {
    fn default() -> Self {
        Self {
            index_file: Cow::Borrowed("./index.html"),
            static_resources_mount: Cow::Borrowed("/"),
            static_resources_location: Cow::Borrowed("./"),
        }
    }
}

/// Helper function to create a default `Spa` instance
pub fn spa() -> Spa {
    Spa::default()
}
