use std::sync::{Arc, Mutex};
use warp::{http::StatusCode, Filter};
use warp::http::{HeaderMap, HeaderValue};
use warp::reply::Response;
use crate::app::SharedScreenshot;

fn create_image_response(filepath: String, filename: String) -> Response {
    let contents = std::fs::read(&filepath).unwrap();
    let mut response = Response::new(contents.into());
    let mut target_headers = HeaderMap::new();
    target_headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/force-download"));
    let disposition = format!("attachment; filename=\"{}\"", filename);
    target_headers.insert(
        "Content-Disposition",
        HeaderValue::from_str(&disposition).unwrap());
    let headers = response.headers_mut();
    headers.extend(target_headers);
    response
}

async fn dyn_reply(shared_image: Arc<Mutex<SharedScreenshot>>, word: String)
        -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let target_image = shared_image.lock().unwrap();
    if target_image.uuid == Some(word) {
        let filepath = target_image.filepath.clone().unwrap();
        let filename = target_image.filename.clone().unwrap();
        let response = create_image_response(filepath, filename);
        Ok(Box::new(response))
    } else {
        Ok(Box::new(StatusCode::NOT_FOUND))
    }
}

pub async fn run(shared_image: Arc<Mutex<SharedScreenshot>>, port: u16) {
    let routes = warp::path::param()
        .and_then(move |word| {
            let image = shared_image.clone();
            dyn_reply(image, word)
        });
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}