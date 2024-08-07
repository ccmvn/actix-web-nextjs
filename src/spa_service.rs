use std::sync::Arc;

use actix_files::Files;
use actix_service::fn_service;
use actix_web::dev::{HttpServiceFactory, ResourceDef, ServiceRequest};
use path_tree::PathTree;
use tracing::debug;

use crate::utils::serve_index;

/// Struct to represent the finalized SPA service
#[derive(Debug)]
pub struct SpaService {
    pub index_file: Arc<String>,
    pub static_resources_location: Arc<String>,
    pub files: Files,
    pub path_tree: Arc<PathTree<String>>,
}

impl HttpServiceFactory for SpaService {
    fn register(self, config: &mut actix_web::dev::AppService) {
        self.files.register(config);

        let path_tree = Arc::clone(&self.path_tree);
        let rdef = ResourceDef::root_prefix("");

        config.register_service(
            rdef,
            None,
            fn_service({
                let index_file = Arc::clone(&self.index_file);
                let static_resources_location = Arc::clone(&self.static_resources_location);

                move |req: ServiceRequest| {
                    let index_file = Arc::clone(&index_file);
                    let static_resources_location = Arc::clone(&static_resources_location);
                    let path_tree = Arc::clone(&path_tree);

                    async move {
                        debug!("Received request for path: {}", req.path());
                        serve_index(req, &index_file, &static_resources_location, &path_tree)
                            .await
                            .map_err(actix_web::Error::from)
                    }
                }
            }),
            None,
        );
    }
}
