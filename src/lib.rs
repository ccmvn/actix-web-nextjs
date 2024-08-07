pub use error::SpaError;
pub use spa::{Spa, spa};
pub use spa_service::SpaService;

mod error;
mod spa;
mod spa_service;
mod utils;

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use actix_web::{App, body::MessageBody, dev::ServiceFactory, Error, http::StatusCode, test};
    use actix_web::dev::{ServiceRequest, ServiceResponse};

    use super::*;

    /// Create a test application with SPA service
    fn test_app() -> App<
        impl ServiceFactory<
            ServiceRequest,
            Response=ServiceResponse<impl MessageBody>,
            Config=(),
            InitError=(),
            Error=Error,
        >,
    > {
        App::new().service(
            Spa::default()
                .index_file("./fixtures/001/index.html")
                .static_resources_location("./fixtures/001")
                .finish(),
        )
    }

    /// Test: Returns the index file for root path
    #[actix_web::test]
    async fn returns_index() {
        let app = test::init_service(test_app()).await;

        let req = test::TestRequest::default().to_request();
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), StatusCode::OK);

        let body = test::read_body(res).await;
        let html = from_utf8(&body).unwrap();
        assert!(html.contains("Home page"));
    }

    /// Test: Returns a specific page for a given path
    #[actix_web::test]
    async fn returns_page() {
        let app = test::init_service(test_app()).await;

        let req = test::TestRequest::default().uri("/page").to_request();
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), StatusCode::OK);

        let body = test::read_body(res).await;
        let html = from_utf8(&body).unwrap();
        assert!(html.contains("Sample Page"));
    }

    /// Test: Returns an item page for a specific path
    #[actix_web::test]
    async fn returns_item_page() {
        let app = test::init_service(test_app()).await;

        let req = test::TestRequest::default().uri("/dog/items/cat").to_request();
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), StatusCode::OK);

        let body = test::read_body(res).await;
        let html = from_utf8(&body).unwrap();
        assert!(html.contains("Item Page"));
    }

    /// Test: Returns the index file for unknown paths
    #[actix_web::test]
    async fn unknown_page_returns_index() {
        let app = test::init_service(test_app()).await;

        let req = test::TestRequest::default().uri("/fsociety").to_request();
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), StatusCode::OK);

        let body = test::read_body(res).await;
        let html = from_utf8(&body).unwrap();
        assert!(html.contains("Home page"));
    }

    /// Test: Returns static assets correctly
    #[actix_web::test]
    async fn returns_assets() {
        let app = test::init_service(test_app()).await;

        let req = test::TestRequest::default().uri("/next.svg").to_request();
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), StatusCode::OK);

        let body = test::read_body(res).await;
        let svg = from_utf8(&body).unwrap();
        assert!(svg.contains(r#"<svg xmlns="http://www.w3.org/2000/svg" fill="none""#));
    }

    /// Test: Returns a dynamic numeric page
    #[cfg(feature = "wildcards")]
    #[actix_web::test]
    async fn test_returns_dynamic_numeric_page() {
        let app = test::init_service(test_app()).await;

        let req = test::TestRequest::default().uri("/1/items/1").to_request();
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), StatusCode::OK);

        let body = test::read_body(res).await;
        let html = from_utf8(&body).unwrap();
        assert!(html.contains("Item Page"));
    }

    /// Test: Returns a dynamic character page
    #[cfg(feature = "wildcards")]
    #[actix_web::test]
    async fn test_returns_dynamic_character_page() {
        let app = test::init_service(test_app()).await;

        let req = test::TestRequest::default().uri("/3b2b6d56-e85b-432d-b555-7113b810a3b7/items/1").to_request();
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), StatusCode::OK);

        let body = test::read_body(res).await;
        let html = from_utf8(&body).unwrap();
        assert!(html.contains("Item Page"));
    }

    /// Test: Handles build manifest not found
    #[cfg(feature = "wildcards")]
    #[actix_web::test]
    async fn handles_build_manifest_not_found() {
        let app = test::init_service(
            App::new().service(
                Spa::default()
                    .index_file("./fixtures/001/index.html")
                    .static_resources_location("./fixtures/no_manifest")
                    .finish(),
            )
        ).await;

        let req = test::TestRequest::default().uri("/").to_request();
        let res = test::call_service(&app, req).await;

        assert_eq!(res.status(), StatusCode::OK);

        let body = test::read_body(res).await;
        let html = from_utf8(&body).unwrap();
        assert!(html.contains("Home page"));
    }
}
