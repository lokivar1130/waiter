use std::{
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
};

use clap::Parser;

#[derive(Parser)]
#[command(name = "serve")]
#[command(version = "0.0.1")]
#[command(about = "Starts a local server and prints incoming requests", long_about = None)]
struct Arguments {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(long, default_value = "8080")]
    port: u16,
    #[arg(long, default_value = "200")]
    return_code: u16,
}

fn main() {
    let args = Arguments::parse();
    let host_and_port = format!("{}:{}", args.host, args.port);
    let http_response_code = match HttpStatusCode::from_u16(args.return_code){
          Some(t) => t,
          None => panic!("Invalid return code {}",args.return_code)
    };

    let listener = match TcpListener::bind(&host_and_port) {
        Ok(listener) => {
            println!("Listening on {}...", host_and_port);
            listener
        }
        Err(e) => {
            println!("Could not bind on {} reason {}", &host_and_port, e);
            return;
        }
    };
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream,http_response_code);
    }
}

fn handle_connection(mut stream: TcpStream, return_code :HttpStatusCode) {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut headers = Vec::new();
    let mut line = String::new();
    loop {
        line.clear();
        match buf_reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                if line == "\r\n" {
                    break;
                }
                headers.push(line.trim().to_string());
            }
            Err(e) => {
                println!("Error reading line: {}", e);
                return;
            }
        }
    }

    println!("Headers: {:#?}", headers);

    let is_chunked = headers.iter().any(|header| {
        header.to_lowercase().starts_with("transfer-encoding:") &&
            header.to_lowercase().contains("chunked")
    });
    let mut body = String::new();
    if is_chunked {
        loop {
            line.clear();
            buf_reader.read_line(&mut line).unwrap();
            let chunk_size_hex = line.trim();
            let chunk_size = usize::from_str_radix(chunk_size_hex, 16).unwrap_or(0);

            if chunk_size == 0 {
                break;
            }

            let mut chunk = vec![0; chunk_size];
            buf_reader.read_exact(&mut chunk).unwrap();
            body.push_str(&String::from_utf8_lossy(&chunk));
            buf_reader.read_line(&mut line).unwrap();
        }
    } else {
        let content_length = headers
            .iter()
            .find(|header| header.to_lowercase().starts_with("content-length:"))
            .and_then(|header| header.split(": ").nth(1))
            .and_then(|value| value.trim().parse::<usize>().ok());
        if let Some(length) = content_length {
            let mut body_vec = vec![0; length];
            buf_reader.read_exact(&mut body_vec).unwrap();
            body = String::from_utf8(body_vec).unwrap();
        }
    }
    println!("Body: {}", body);
    let response = format!("HTTP/1.1 {} {}\r\n\r\n", return_code as i32, return_code.reason_phrase());
    stream.write_all(response.as_bytes()).unwrap();
}
#[derive(Debug, Clone, Copy)]
pub enum HttpStatusCode {
    Continue = 100,
    SwitchingProtocols = 101,
    Processing = 102,
    OK = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,
    MultiStatus = 207,
    AlreadyReported = 208,
    IMUsed = 226,
    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    UseProxy = 305,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    PayloadTooLarge = 413,
    URITooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    ImATeapot = 418,
    MisdirectedRequest = 421,
    UnprocessableEntity = 422,
    Locked = 423,
    FailedDependency = 424,
    TooEarly = 425,
    UpgradeRequired = 426,
    PreconditionRequired = 428,
    TooManyRequests = 429,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HTTPVersionNotSupported = 505,
    VariantAlsoNegotiates = 506,
    InsufficientStorage = 507,
    LoopDetected = 508,
    NotExtended = 510,
    NetworkAuthenticationRequired = 511,
}

impl HttpStatusCode {
    pub fn reason_phrase(&self) -> &'static str {
        match self {
            HttpStatusCode::Continue => "Continue",
            HttpStatusCode::SwitchingProtocols => "Switching Protocols",
            HttpStatusCode::Processing => "Processing",
            HttpStatusCode::OK => "OK",
            HttpStatusCode::Created => "Created",
            HttpStatusCode::Accepted => "Accepted",
            HttpStatusCode::NonAuthoritativeInformation => "Non-Authoritative Information",
            HttpStatusCode::NoContent => "No Content",
            HttpStatusCode::ResetContent => "Reset Content",
            HttpStatusCode::PartialContent => "Partial Content",
            HttpStatusCode::MultiStatus => "Multi-Status",
            HttpStatusCode::AlreadyReported => "Already Reported",
            HttpStatusCode::IMUsed => "IM Used",
            HttpStatusCode::MultipleChoices => "Multiple Choices",
            HttpStatusCode::MovedPermanently => "Moved Permanently",
            HttpStatusCode::Found => "Found",
            HttpStatusCode::SeeOther => "See Other",
            HttpStatusCode::NotModified => "Not Modified",
            HttpStatusCode::UseProxy => "Use Proxy",
            HttpStatusCode::TemporaryRedirect => "Temporary Redirect",
            HttpStatusCode::PermanentRedirect => "Permanent Redirect",
            HttpStatusCode::BadRequest => "Bad Request",
            HttpStatusCode::Unauthorized => "Unauthorized",
            HttpStatusCode::PaymentRequired => "Payment Required",
            HttpStatusCode::Forbidden => "Forbidden",
            HttpStatusCode::NotFound => "Not Found",
            HttpStatusCode::MethodNotAllowed => "Method Not Allowed",
            HttpStatusCode::NotAcceptable => "Not Acceptable",
            HttpStatusCode::ProxyAuthenticationRequired => "Proxy Authentication Required",
            HttpStatusCode::RequestTimeout => "Request Timeout",
            HttpStatusCode::Conflict => "Conflict",
            HttpStatusCode::Gone => "Gone",
            HttpStatusCode::LengthRequired => "Length Required",
            HttpStatusCode::PreconditionFailed => "Precondition Failed",
            HttpStatusCode::PayloadTooLarge => "Payload Too Large",
            HttpStatusCode::URITooLong => "URI Too Long",
            HttpStatusCode::UnsupportedMediaType => "Unsupported Media Type",
            HttpStatusCode::RangeNotSatisfiable => "Range Not Satisfiable",
            HttpStatusCode::ExpectationFailed => "Expectation Failed",
            HttpStatusCode::ImATeapot => "I'm a teapot",
            HttpStatusCode::MisdirectedRequest => "Misdirected Request",
            HttpStatusCode::UnprocessableEntity => "Unprocessable Entity",
            HttpStatusCode::Locked => "Locked",
            HttpStatusCode::FailedDependency => "Failed Dependency",
            HttpStatusCode::TooEarly => "Too Early",
            HttpStatusCode::UpgradeRequired => "Upgrade Required",
            HttpStatusCode::PreconditionRequired => "Precondition Required",
            HttpStatusCode::TooManyRequests => "Too Many Requests",
            HttpStatusCode::RequestHeaderFieldsTooLarge => "Request Header Fields Too Large",
            HttpStatusCode::UnavailableForLegalReasons => "Unavailable For Legal Reasons",
            HttpStatusCode::InternalServerError => "Internal Server Error",
            HttpStatusCode::NotImplemented => "Not Implemented",
            HttpStatusCode::BadGateway => "Bad Gateway",
            HttpStatusCode::ServiceUnavailable => "Service Unavailable",
            HttpStatusCode::GatewayTimeout => "Gateway Timeout",
            HttpStatusCode::HTTPVersionNotSupported => "HTTP Version Not Supported",
            HttpStatusCode::VariantAlsoNegotiates => "Variant Also Negotiates",
            HttpStatusCode::InsufficientStorage => "Insufficient Storage",
            HttpStatusCode::LoopDetected => "Loop Detected",
            HttpStatusCode::NotExtended => "Not Extended",
            HttpStatusCode::NetworkAuthenticationRequired => "Network Authentication Required",
        }
    }
    pub fn from_u16(value: u16) -> Option<HttpStatusCode> {
        match value {
            100 => Some(HttpStatusCode::Continue),
            101 => Some(HttpStatusCode::SwitchingProtocols),
            102 => Some(HttpStatusCode::Processing),
            200 => Some(HttpStatusCode::OK),
            201 => Some(HttpStatusCode::Created),
            202 => Some(HttpStatusCode::Accepted),
            203 => Some(HttpStatusCode::NonAuthoritativeInformation),
            204 => Some(HttpStatusCode::NoContent),
            205 => Some(HttpStatusCode::ResetContent),
            206 => Some(HttpStatusCode::PartialContent),
            207 => Some(HttpStatusCode::MultiStatus),
            208 => Some(HttpStatusCode::AlreadyReported),
            226 => Some(HttpStatusCode::IMUsed),
            300 => Some(HttpStatusCode::MultipleChoices),
            301 => Some(HttpStatusCode::MovedPermanently),
            302 => Some(HttpStatusCode::Found),
            303 => Some(HttpStatusCode::SeeOther),
            304 => Some(HttpStatusCode::NotModified),
            305 => Some(HttpStatusCode::UseProxy),
            307 => Some(HttpStatusCode::TemporaryRedirect),
            308 => Some(HttpStatusCode::PermanentRedirect),
            400 => Some(HttpStatusCode::BadRequest),
            401 => Some(HttpStatusCode::Unauthorized),
            402 => Some(HttpStatusCode::PaymentRequired),
            403 => Some(HttpStatusCode::Forbidden),
            404 => Some(HttpStatusCode::NotFound),
            405 => Some(HttpStatusCode::MethodNotAllowed),
            406 => Some(HttpStatusCode::NotAcceptable),
            407 => Some(HttpStatusCode::ProxyAuthenticationRequired),
            408 => Some(HttpStatusCode::RequestTimeout),
            409 => Some(HttpStatusCode::Conflict),
            410 => Some(HttpStatusCode::Gone),
            411 => Some(HttpStatusCode::LengthRequired),
            412 => Some(HttpStatusCode::PreconditionFailed),
            413 => Some(HttpStatusCode::PayloadTooLarge),
            414 => Some(HttpStatusCode::URITooLong),
            415 => Some(HttpStatusCode::UnsupportedMediaType),
            416 => Some(HttpStatusCode::RangeNotSatisfiable),
            417 => Some(HttpStatusCode::ExpectationFailed),
            418 => Some(HttpStatusCode::ImATeapot),
            421 => Some(HttpStatusCode::MisdirectedRequest),
            422 => Some(HttpStatusCode::UnprocessableEntity),
            423 => Some(HttpStatusCode::Locked),
            424 => Some(HttpStatusCode::FailedDependency),
            425 => Some(HttpStatusCode::TooEarly),
            426 => Some(HttpStatusCode::UpgradeRequired),
            428 => Some(HttpStatusCode::PreconditionRequired),
            429 => Some(HttpStatusCode::TooManyRequests),
            431 => Some(HttpStatusCode::RequestHeaderFieldsTooLarge),
            451 => Some(HttpStatusCode::UnavailableForLegalReasons),
            500 => Some(HttpStatusCode::InternalServerError),
            501 => Some(HttpStatusCode::NotImplemented),
            502 => Some(HttpStatusCode::BadGateway),
            503 => Some(HttpStatusCode::ServiceUnavailable),
            504 => Some(HttpStatusCode::GatewayTimeout),
            505 => Some(HttpStatusCode::HTTPVersionNotSupported),
            506 => Some(HttpStatusCode::VariantAlsoNegotiates),
            507 => Some(HttpStatusCode::InsufficientStorage),
            508 => Some(HttpStatusCode::LoopDetected),
            510 => Some(HttpStatusCode::NotExtended),
            511 => Some(HttpStatusCode::NetworkAuthenticationRequired),
            _ => None,
        }
    }
}

