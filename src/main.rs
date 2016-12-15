#[macro_use]
extern crate nickel;
extern crate multipart;
extern crate hyper;

use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs::{copy, remove_file};
use std::collections::HashMap;
use hyper::header::Location;
use multipart::server::{Entries, Multipart, SaveResult};
use nickel::{HttpRouter, MiddlewareResult, Nickel, Request, Response, Options, StaticFilesHandler,
             QueryString};
use nickel::status::StatusCode;

const MAX_UPLOADED_SIZE: u64 = 2 * 1024 * 1024;
const UPLOAD_PATH: &'static str = "data/images";

fn is_image(path: &PathBuf) -> bool {
    return match Command::new("file").arg("-ib").arg(path.as_path().to_str().unwrap()).output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout).contains("image/"),
        Err(_) => false,
    };
}

fn process_uploaded_file<'a>(path: &PathBuf, name: &str) -> Result<String, String> {
    let data = Path::new(UPLOAD_PATH);
    let name_path = Path::new(name);
    if !path.exists() {
        return Err(String::from("Invalid upload path"));
    }

    if !is_image(path) {
        return Err(format!("File {} is not an image", name));
    }

    if let Ok(cmd) = Command::new("sha256sum")
        .arg(path.as_path()
            .to_str()
            .unwrap())
        .output() {
        let d = String::from_utf8_lossy(&cmd.stdout);
        if let Some(digest) = d.split(' ').next() {
            if let Some(ext) = name_path.extension() {
                let name = format!("{}.{}", digest, ext.to_str().unwrap());
                let fp = data.join(&name[..2]).join(&name);
                Command::new("mkdir")
                    .arg("-p")
                    .arg(data.join(&name[..2]))
                    .output()
                    .expect(&format!("Unable to mkdir for {:?}", fp));
                copy(path, &fp).unwrap();
                remove_file(path).unwrap();
                Ok(String::from(name))
            } else {
                return Err(format!("File '{}' has no extension", name));
            }
        } else {
            return Err(format!("Invalid checksum value: {}", d));
        }
    } else {
        return Err(String::from("Unable to checksum"));
    }
}

fn handle_multipart<'mw>(req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
    match Multipart::from_request(req) {
        Ok(mut multipart) => {
            match multipart.save_all() {
                SaveResult::Full(entries) => process_entries(res, entries),
                SaveResult::Partial(entries, e) => {
                    println!("Partial errors ... {:?}", e);
                    return process_entries(res, entries);
                }
                SaveResult::Error(e) => {
                    println!("Error in multipart POST ... {:?}", e);
                    res.set(nickel::status::StatusCode::InternalServerError);
                    return res.send(format!("Server could not handle multipart POST! {:?}", e));
                }
            }
        }
        Err(_) => {
            res.set(nickel::status::StatusCode::BadRequest);
            return res.send("Invalid multi-part request");
        }
    }
}

fn process_entries<'mw>(mut res: Response<'mw>, entries: Entries) -> MiddlewareResult<'mw> {
    if entries.files.len() > 1 {
        res.set(nickel::status::StatusCode::BadRequest);
        return res.send("Too many files, only 1 is allowed");
    }

    let mut uploaded = Vec::new();
    let mut error = None;
    for (name, savedfile) in entries.files {
        if savedfile.size > MAX_UPLOADED_SIZE {
            error = Some(format!("File {} is too big: {} bytes, max allowed size is {} bytes",
                                 name,
                                 savedfile.size,
                                 MAX_UPLOADED_SIZE));
            break;
        }
        if let Some(file_name) = savedfile.filename {
            let processed = process_uploaded_file(&savedfile.path, &file_name);
            if let Ok(file) = processed {
                uploaded.push(file);
            } else if let Err(err) = processed {
                error = Some(err);
                break;
            }
        } else {
            error = Some(format!("Invalid file name: {}", name));
            break;
        }
    }

    if error.is_none() {
        res.set(Location(format!("/image/?id={}", &uploaded[0])));
        res.set(StatusCode::MovedPermanently);
        res.send("")
    } else {
        res.set(nickel::status::StatusCode::BadRequest);
        res.send(error.unwrap())
    }
}

fn upload_form<'mw>(_: &mut Request, res: Response<'mw>) -> MiddlewareResult<'mw> {
    let map = HashMap::<&str, &str>::new();
    res.render("data/index.html", &map)
}

fn get_image<'mw>(req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
    let id = req.query().get("id").unwrap();
    if id == "" {
        res.set(StatusCode::NotFound);
        return res.send("<h1>Page not found</h1>");
    }

    let path = format!("{}/{}", &id[..2], &id);
    let mut map = HashMap::<&str, &str>::new();
    map.insert("digest", id);
    map.insert("path", &path);
    res.render("data/image.html", &map)
}

fn main() {
    let mut srv = Nickel::new();

    srv.utilize(StaticFilesHandler::new("data/"));

    srv.get("/", upload_form);
    srv.get("/image/", get_image);
    srv.post("/upload/", handle_multipart);

    srv.options = Options::default()
        .output_on_listen(false)
        .thread_count(Some(8));
    srv.listen("0.0.0.0:6868").expect("Failed to bind server");
}
