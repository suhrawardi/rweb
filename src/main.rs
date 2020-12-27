use std::net::SocketAddr;
use futures_util::{stream, StreamExt};
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
use hyper::body::Buf;
use hyper::client::HttpConnector;
use hyper::service::{make_service_fn, service_fn};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

extern crate pretty_env_logger;
#[macro_use] extern crate log;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;

static HTML_PATH: &str = "src/html/";
static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
static NOTFOUND: &[u8] = b"Not Found";
static POST_DATA: &str = r#"{"original": "data"}"#;
static URL: &str = "http://127.0.0.1:3000/json_api";

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
    debug!("Listening on http://{}", addr);
    server.await?;
    Ok(())
}

async fn req_handler(
    req: Request<Body>,
    client: Client<HttpConnector>,
) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => {
            simple_file_send("index.html").await
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
    let req = Request::builder()
        .method(Method::POST)
        .uri(URL)
        .header(header::CONTENT_TYPE, "application/json")
        .body(POST_DATA.into())
        .unwrap();
    let web_res = client.request(req).await?;
    let before = stream::once(async {
        Ok(format!(
            "<p><b>POST request body</b>:</p>{}<br/><br/><b>Response</></b>:</p>",
            POST_DATA
        )
        .into())
    });
    let after = web_res.into_body();
    let body = Body::wrap_stream(before.chain(after));
    Ok(Response::new(body))
}

async fn api_post_resp(req: Request<Body>) -> Result<Response<Body>> {
    let whole_body = hyper::body::aggregate(req).await?;
    let mut data: serde_json::Value = serde_json::from_reader(whole_body.reader())?;
    data["test"] = serde_json::Value::from("test_value");
    let json = serde_json::to_string(&data)?;
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json))?;
    Ok(response)
}

async fn api_get_resp() -> Result<Response<Body>> {
    let data = vec!["foo", "bar"];
    let res = match serde_json::to_string(&data) {
        Ok(json) => Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(INTERNAL_SERVER_ERROR.into())
            .unwrap(),
    };
    Ok(res)
}

async fn simple_file_send(filename: &str) -> Result<Response<Body>> {
    let mut path: String = HTML_PATH.to_owned();
    path.push_str(filename);
    if let Ok(file) = File::open(path).await {
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
