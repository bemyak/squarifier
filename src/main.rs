use std::cmp::{max, min};

use http_types::{mime, Mime, Request, Response, StatusCode, Url};
use image::{DynamicImage, ImageError, ImageFormat, RgbaImage};
use tide::Request as TideReq;

const PAGE: &str = include_str!("site/index.html");
const FAVICON: &[u8] = include_bytes!("site/favicon.ico");

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    tide::log::start();
    let mut app = tide::new();
    app.at("/").get(|_| async {
        let mut res = Response::new(StatusCode::Ok);
        res.set_content_type(mime::HTML);
        res.set_body(PAGE);
        Ok(res)
    });
    app.at("/favicon.ico").get(|_| async {
        let mut res = Response::new(StatusCode::Ok);
        res.set_content_type(mime::ICO);
        res.set_body(FAVICON);
        Ok(res)
    });
    app.at("/:param").get(|req: TideReq<()>| async move {
        let err = process_req(req.into()).await;
        match err {
            Ok(r) => Ok(r),
            Err(r) => Ok(r),
        }
    });
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

type Err = Result<Response, Response>;

async fn process_req(req: Request) -> Err {
    let param = req.url().path().trim_start_matches('/');

    let (_, encoded_url) = {
        let mut i = param.rsplitn(2, '.');
        (i.next().unwrap_or_default(), i.next().unwrap_or_default())
    };

    let decoded = base64::decode(encoded_url).map_err(|_| {
        let mut res = Response::new(StatusCode::BadRequest);
        res.append_header("error", "Can't decode base64 data");
        res
    })?;
    let decoded_str = std::str::from_utf8(&decoded).map_err(|_| {
        let mut res = Response::new(StatusCode::BadRequest);
        res.append_header("error", "Can't decode base64 data");
        res
    })?;
    let url = Url::parse(decoded_str).map_err(|err| {
        let mut res = Response::new(StatusCode::BadRequest);
        let error = format!("Can't parse URL: {}", err.to_string());
        let error: &str = error.as_ref();
        res.append_header("error", error);
        res
    })?;

    let mut res1 = surf::get(url.clone()).await.map_err(|err| {
        let mut res = Response::new(StatusCode::InternalServerError);
        let error = format!("Can't fetch url {}: {}", url, err.to_string());
        let error: &str = error.as_ref();
        res.append_header("error", error);
        res
    })?;

    if !res1.status().is_success() {
        let mut res = Response::new(res1.status());
        res.set_body(res1.body_bytes().await.unwrap_or_default());
        return Err(res);
    }

    let mime = res1.content_type().unwrap_or(mime::PLAIN);
    if mime.basetype() != "image" {
        let mut res = Response::new(StatusCode::BadRequest);
        let error = format!(
            "Bad content type for {}, expected image, got: {}",
            url, mime
        );
        let error: &str = error.as_ref();
        res.append_header("error", error);
        return Err(res);
    }

    let bytes = res1.body_bytes().await.map_err(|err| {
        let mut res = Response::new(StatusCode::BadRequest);
        let error = format!("Cannot read data from {}: {}", url, err);
        let error: &str = error.as_ref();
        res.append_header("error", error);
        res
    })?;

    let format = mime_to_image_format(&mime, &url).map_err(|err| {
        let mut res = Response::new(StatusCode::BadRequest);
        let error = format!("Unsupported image type for {}: {}", url, err);
        let error: &str = error.as_ref();
        res.append_header("error", error);
        res
    })?;

    let result = squarify(&bytes, format).map_err(|err| {
        let mut res = Response::new(StatusCode::BadRequest);
        let error = format!("Failed to decode image {}: {}", url, err);
        let error: &str = error.as_ref();
        res.append_header("error", error);
        res
    })?;

    let mut res = Response::new(StatusCode::Ok);
    res.set_body(result);
    res.set_content_type(mime);
    Ok(res)
}

fn mime_to_image_format(mime: &Mime, url: &Url) -> Result<ImageFormat, ImageError> {
    match mime.subtype() {
        "png" => Ok(ImageFormat::Png),
        "jpeg" | "jpg" => Ok(ImageFormat::Jpeg),
        "gif" => Ok(ImageFormat::Gif),
        "webp" => Ok(ImageFormat::WebP),
        "pnm" => Ok(ImageFormat::Pnm),
        "tiff" => Ok(ImageFormat::Tiff),
        "tga" => Ok(ImageFormat::Tga),
        "dds" => Ok(ImageFormat::Dds),
        "bpm" => Ok(ImageFormat::Bmp),
        "ico" => Ok(ImageFormat::Ico),
        "hdr" => Ok(ImageFormat::Hdr),
        "farbfeld" => Ok(ImageFormat::Farbfeld),
        "avif" => Ok(ImageFormat::Avif),
        _ => ImageFormat::from_path(url.to_string()),
    }
}

fn squarify(bytes: &[u8], format: ImageFormat) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let src_image = image::load_from_memory_with_format(&bytes, format)?;
    let src_rgba = src_image.to_rgba8();

    let mut min_x = src_rgba.width();
    let mut max_x = 0;
    let mut min_y = src_rgba.height();
    let mut max_y = 0;

    for (x, y, pixel) in src_rgba.enumerate_pixels() {
        if pixel[3] != 0 {
            max_x = max(x, max_x);
            min_x = min(x, min_x);
            max_y = max(y, max_y);
            min_y = min(y, min_y);
        }
    }

    let cropped_width = max_x - min_x;
    let cropped_height = max_y - min_y;

    let new_dimensions = max(cropped_width, cropped_height);
    let shift_x = (new_dimensions - cropped_width) / 2;
    let shift_y = (new_dimensions - cropped_height) / 2;

    let mut new_image = RgbaImage::new(new_dimensions, new_dimensions);

    for y in 0..cropped_height {
        for x in 0..cropped_width {
            let pixel = *src_rgba.get_pixel(x + min_x, y + min_y);
            new_image.put_pixel(x + shift_x, y + shift_y, pixel);
        }
    }

    let mut buff = Vec::new();
    DynamicImage::ImageRgba8(new_image).write_to(&mut buff, format)?;
    Ok(buff)
}
