use warp::Filter;
use std::path::Path;
use std::fs;
use rand::seq::SliceRandom;
use std::sync::Arc;
use tokio::sync::Mutex;
use mime_guess::from_path;
use warp::http::header::HeaderValue;
use warp::reply::Response;
use log;
use env_logger;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Scan all in 'files' directory
    let files_dir = Path::new("files");
    let files = fs::read_dir(files_dir)
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

    // Print the list of files
    log::info!("Files found in 'files' directory: {:?}", files);

    if files.is_empty() {
        eprintln!("No files found in the 'files' directory.");
        return;
    }

    // Wrapping file lists in Arc and Mutex for sharing in multi-threaded environments
    let files = Arc::new(Mutex::new(files));

    // Create a Warp Filter to process any requests
    let route = warp::any()
        .and(warp::path::end())
        .and_then(move || {
            let files = files.clone();
            async move {
                let files = files.lock().await;
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
        });

    // Start Web service
    warp::serve(route)
        .run(([0, 0, 0, 0], 3030))
        .await;
}
