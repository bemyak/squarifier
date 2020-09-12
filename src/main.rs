use surf::url::Url;
use tide::{Request, Response, Result as TideResult, StatusCode};

const PAGE: &'static str = include_str!("site/index.html");
const FAVICON: &'static [u8] = include_bytes!("site/favicon.ico");

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    // let page = include_str!("site/index.html");

    tide::log::start();
    let mut app = tide::new();
    app.at("/").get(|_| async {
        let mut res = Response::new(StatusCode::Ok);
        res.set_content_type("text/html");
        res.set_body(PAGE);
        Ok(res)
    });
    app.at("/favicon.ico").get(|_| async {
        let mut res = Response::new(StatusCode::Ok);
        res.set_content_type("mage/vnd.microsoft.icon");
        res.set_body(FAVICON);
        Ok(res)
    });
    app.at("/:param")
        .get(|req: Request<()>| async move { process_req(req).await });
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn process_req(req: Request<()>) -> TideResult {
    let param: Result<String, _> = req.param("param");
    let res = match param {
        Err(_) => {
            let mut res = Response::new(StatusCode::BadRequest);
            res.append_header("error", "You need to specify target url as parameter");
            res
        }
        Ok(param) => {
            let decoded = urldecode::decode(param);
            let url = Url::parse(&decoded);
            match url {
                Err(err) => {
                    let mut res = Response::new(StatusCode::BadRequest);
                    let error = format!("Can't parse URL: {}", err.to_string());
                    let error: &str = error.as_ref();
                    res.append_header("error", error);
                    res
                }
                Ok(url) => {
                    let result = surf::get(url.clone()).await;
                    match result {
                        Err(err) => {
                            let mut res = Response::new(StatusCode::InternalServerError);
                            let error = format!("Can't fetch url {}: {}", url, err.to_string());
                            let error: &str = error.as_ref();
                            res.append_header("error", error);
                            res
                        }
                        Ok(res1) => {
                            // Response {
                            //     response: Response {
                            //         status: 200,
                            //         version: HTTP/2.0,
                            //         headers: {
                            //             "content-type": "image/png",
                            //             "last-modified": "Sun, 30 Aug 2020 08:28:48 GMT",
                            //             "accept-ranges": "bytes",
                            //             "etag": "\"8e2a4495a77ed61:0\"",
                            //             "server": "Microsoft-IIS/10.0",
                            //             "x-powered-by": "ASP.NET",
                            //             "date": "Sat, 12 Sep 2020 17:13:00 GMT",
                            //             "content-length": "341002",
                            //         },
                            //         body: Body {
                            //             reader: "<hidden>",
                            //         },
                            //     },
                            // }
                            dbg!(res1);
                            Response::new(StatusCode::Ok)
                        }
                    }
                }
            }
        }
    };
    Ok(res)
}
