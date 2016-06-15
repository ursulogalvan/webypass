use std::io::{Read, Write};

use hyper::Url;
use hyper::uri:: RequestUri;
use hyper::header::Headers;
use hyper::client::Client;
use hyper::server::{Server, Request, Response};

extern crate hyper;

fn proxy(req: Request, mut res: Response, target_url: &str) {
    let client = Client::new();
    let mut proxied_res = client.request(req.method, target_url).headers(req.headers).send().unwrap();
    let mut buffer = Vec::new();
    let _ = proxied_res.read_to_end(&mut buffer);
    {
        let h: &mut Headers = res.headers_mut();
        *h = proxied_res.headers.clone();
    }
    let mut res = res.start().unwrap();
    res.write_all(&buffer).unwrap();
}

fn get_url_in_query(url: &Url) -> Option<String> {
    let find_res = url.query_pairs().find(|&(ref k, _)| k == "url");
    if let Some((_, qurl)) = find_res {
        Some(qurl.as_ref().to_string())
    } else {
        None
    }
}

fn hello(req: Request, mut res: Response) {
    let uri = req.uri.clone();
    match uri {
        RequestUri::AbsoluteUri(url) => {
            println!("URI: {}", url);
            if let Some(target_url) = get_url_in_query(&url) {
                proxy(req, res, &target_url);
                return;
            }
        },
        RequestUri::AbsolutePath(path) => {
            println!("Path {}", path);
            let url = Url::parse("http://localhost").unwrap().join(&path).unwrap();
            if let Some(target_url) = get_url_in_query(&url) {
                proxy(req, res, &target_url);
                return;
            }
        },
        _ => {
            println!("Unsupported URI type");
        }
    }
    *res.status_mut() = hyper::NotFound;
    res.send(b"Not Found").unwrap();
}

fn main() {
    Server::http("127.0.0.1:8080").unwrap().handle(hello).unwrap();
}

