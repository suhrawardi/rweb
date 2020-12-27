use std::net::SocketAddr;
use hyper::{Body, Error, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};


static INDEX: &str = "examples/send_file_index.html";
static NOTFOUND: &[u8] = b"Not Found";


#[tokio::main]
pub async fn main() {
    pretty_env_logger::init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_service = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(microservice_handler))
    });

    let server = Server::bind(&addr).serve(make_service);

    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn microservice_handler(req: Request<Body>) -> Result<Response<Body>, Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => simple_file_send(INDEX).await,
        (&Method::GET, "/no_file.html") => {
            simple_file_send("this_file_should_not_exist.html").await
        }
        _ => Ok(not_found()),
    }
}

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(NOTFOUND.into())
        .unwrap()
}

async fn simple_file_send(filename: &str) -> Result<Response<Body>, Error> {
    if let Ok(file) = File::open(filename).await {
        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(stream);
        return Ok(Response::new(body));
    }
    Ok(not_found())
}
