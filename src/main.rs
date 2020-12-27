use std::net::SocketAddr;
use hyper::client::HttpConnector;
use hyper::{
    header,
    Body,
    Client,
    Method,
    Request,
    Response,
    Server,
    StatusCode
};
use hyper::service::{make_service_fn, service_fn};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;

static INDEX: &str = "examples/send_file_index.html";
static NOTFOUND: &[u8] = b"Not Found";


#[tokio::main]
pub async fn main() -> Result<()> {
    pretty_env_logger::init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let client = Client::new();

    let make_service = make_service_fn(move |_conn| {
        let client = client.clone();
        async {
            Ok::<_, GenericError>(service_fn(move |req| {
                req_handler(req, client.to_owned())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    println!("Listening on http://{}", addr);

    server.await?;
    Ok(())
}

async fn req_handler(
    req: Request<Body>,
    client: Client<HttpConnector>,
) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => {
            simple_file_send(INDEX).await
        },
        (&Method::GET, "/test.html") => client_req_resp(&client).await,
        (&Method::POST, "/json_api") => api_post_resp(req).await,
        (&Method::GET, "/json_api") => api_get_resp().await,
        (&Method::GET, "/no_file.html") => {
            simple_file_send("this_file_should_not_exist.html").await
        }
        _ => Ok(not_found()),
    }
}

async fn client_req_resp(client: &Client<HttpConnector>) -> Result<Response<Body>> {
    Ok(not_found())
}

async fn api_post_resp(req: Request<Body>) -> Result<Response<Body>> {
    Ok(not_found())
}

async fn api_get_resp() -> Result<Response<Body>> {
    Ok(not_found())
}

async fn simple_file_send(filename: &str) -> Result<Response<Body>> {
    if let Ok(file) = File::open(filename).await {
        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(stream);
        return Ok(Response::new(body));
    }
    Ok(not_found())
}

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(NOTFOUND.into())
        .unwrap()
}
