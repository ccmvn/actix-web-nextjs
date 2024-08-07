# actix-web-nextjs

Actix Web service for hosting [statically exported](https://nextjs.org/docs/app/building-your-application/deploying/static-exports) Next.js apps. This is a fork of the [SPA feature from actix-web-lab](https://docs.rs/actix-web-lab/0.21.0/actix_web_lab/web/fn.spa.html) with added support for Next.js dynamic routes.

## Overview

The `actix-web-nextjs` service makes it easy to serve statically exported Next.js applications. Key features include:

- **Index File Fallback**: Automatically serves a specified index file for unknown routes, ensuring seamless SPA navigation.
- **Static File Handling**: Efficiently serves static assets from a specified directory.
- **Dynamic Route Support**: Converts Next.js dynamic routes like `/pet/dog/husky` into `/pet/[petType]/[breed].html`.

## How it works

The service searches for the `_buildManifest.js` file generated by Next.js and builds a tree of routes from it. Requests like `/pet/dog/husky` resolve into `/pet/[petType]/[breed].html`. When a new request with a specific route is made, the service attempts to find and serve the corresponding file based on the dynamic route structure. If the file is not found, it defaults to serving the index file.

## Sample Usage

```rust
use actix_web::App;
use actix_web_nextjs::spa;

let app = App::new()
    // API routes and other services
    .service(
        spa()
            .index_file("dist/index.html")
            .static_resources_mount("dist")
            .static_resources_location("/")
            .finish()
    );
```

## How to Install

Add `actix-web-nextjs` to your dependencies:

```toml
[dependencies]
actix-web-nextjs = "0.2.3"
```

`actix-web-nextjs` exposes the following feature flags:

- `wildcards`: Enables support for wildcard routes (enabled by default).

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in actix-web-nextjs by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
