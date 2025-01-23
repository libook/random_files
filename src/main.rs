use warp::Filter;
use std::path::{Path, PathBuf};
use std::fs;
use rand::seq::SliceRandom;
use std::sync::Arc;
use tokio::sync::Mutex;
use mime_guess::from_path;
use warp::http::header::HeaderValue;
use warp::reply::Response;
use log;
use env_logger;
use std::collections::HashMap;
use url;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Wrapping file lists in Arc and Mutex for sharing in multi-threaded environments
    // Use a HashMap to cache file lists for different subdirectories
    let files_cache = Arc::new(Mutex::new(HashMap::<String, Vec<PathBuf>>::new()));

    // Helper function to handle file listing and caching
    let handle_request = {
        let files_cache = files_cache.clone();
        move |subdir: String, refresh_cache: bool| {
            let files_cache = files_cache.clone();
            async move {
                let mut cache = files_cache.lock().await;

                // If refresh_cache is true or cache doesn't contain this subdir, refresh the cache
                if refresh_cache || !cache.contains_key(&subdir) {
                    let files_dir = Path::new("files").join(&subdir);
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

                    if files.is_empty() {
                        return Err(warp::reject::not_found());
                    }

                    // Update the cache for this subdirectory
                    cache.insert(subdir.clone(), files);
                }

                // Get the cached files for this subdirectory
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

    // Route for both root and subdirectories
    let route = warp::path::tail()
        .and(warp::filters::query::raw()
            .or_else(|_| async { Ok::<(String,), warp::Rejection>((String::new(),)) }) // Handle empty query string
            .map(|query: String| {
                log::info!("Raw query string: {}", query); // Debug log
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
                log::info!("Parsed refresh_cache: {}", refresh_cache); // Debug log
                refresh_cache
            }))
        .and_then(move |path: warp::path::Tail, refresh_cache: bool| {
            let subdir = path.as_str().to_string(); // Get the subdirectory path
            let handle_request = handle_request.clone();
            async move {
                handle_request(subdir, refresh_cache).await
            }
        });

    // Start Web service
    warp::serve(route)
        .run(([0, 0, 0, 0], 3030))
        .await;
}
