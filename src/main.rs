use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use rand::seq::SliceRandom;
use mime_guess::from_path;
use warp::Filter;
use warp::http::header::HeaderValue;
use warp::reply::Response;
use url;
use urlencoding::decode;
use log;
use env_logger;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Read environment variables
    let port: u16 = env::var("LISTEN_PORT")
        .map(|p| p.parse().unwrap_or(3030))
        .unwrap_or(3030);

    // Wrapping file lists in Arc and Mutex for sharing in multi-threaded environments
    // Use a HashMap to cache file lists for different subdirectories
    let files_cache = Arc::new(Mutex::new(HashMap::<String, Vec<PathBuf>>::new()));

    // Request handler
    let handle_request = {
        move |subdir: String, refresh_cache: bool| {
            log::info!("Handling request for subdirectory: {}", subdir);

            let files_cache = files_cache.clone();
            async move {
                let mut cache = files_cache.lock().await;

                // If refresh_cache is true or cache doesn't contain this subdir, refresh the cache
                if refresh_cache || !cache.contains_key(&subdir) {
                    let files_dir = Path::new("files").join(&subdir);
                    if !files_dir.exists() || !files_dir.is_dir() {
                        return Err(warp::reject::not_found());
                    }
                    let files = fs::read_dir(&files_dir)
                        .expect("Failed to read files directory")
                        .filter_map(|entry| {
                            let entry = entry.ok()?;
                            if entry.file_type().ok()?.is_file() {
                                Some(entry.path())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    log::info!("Files found in '{}' directory: {:?}", files_dir.display(), files);

                    // 404 for empty directories
                    if files.is_empty() {
                        return Err(warp::reject::not_found());
                    }

                    // Update the file list cache
                    cache.insert(subdir.clone(), files);
                }

                let files = cache.get(&subdir).unwrap();

                // Select a random file
                let file = files.choose(&mut rand::thread_rng()).unwrap();

                // Print the selected file
                log::info!("Selected file: {:?}", file);

                // Read file contents
                let content = match fs::read(file) {
                    Ok(content) => content,
                    Err(_) => return Err(warp::reject::not_found()),
                };

                // Get Content-Type from filename
                let mime_type = from_path(file).first_or_octet_stream();
                let mime_type = HeaderValue::from_str(mime_type.as_ref()).unwrap();

                // Build response
                let mut response = Response::new(content.into());
                response.headers_mut().insert("Content-Type", mime_type);
                Ok::<_, warp::Rejection>(response)
            }
        }
    };

    // Route filters
    let route = warp::path::tail()
        // Process query string
        .and(warp::filters::query::raw()
            // Handle empty query string
            .or_else(|_| async { Ok::<(String,), warp::Rejection>((String::new(),)) })
            // Prepare query parameters
            .map(|query: String| {
                log::info!("Raw query string: {}", query);
                // Parse the query string manually
                let mut refresh_cache = false; // Default to false
                if !query.is_empty() {
                    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
                        if key == "refresh_cache" {
                            refresh_cache = value == "true";
                            break;
                        }
                    }
                }
                log::info!("Parsed refresh_cache: {}", refresh_cache);
                refresh_cache
            }))
        // Process path
        .and_then(move |path: warp::path::Tail, refresh_cache: bool| {
            // Decode the URL-encoded path
            let decoded_path = decode(path.as_str()).unwrap_or_else(|_| path.as_str().to_string().into());
            let subdir = decoded_path.to_string();

            // Call handler
            let handle_request = handle_request.clone();
            async move {
                handle_request(subdir.to_string(), refresh_cache).await
            }
        });

    // Bind and serve
    warp::serve(route)
        .run(([0, 0, 0, 0], port))
        .await;
}
