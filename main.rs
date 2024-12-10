use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use socket2::{Domain, Socket, Type};
use std::net::TcpListener;

#[derive(Serialize)]
struct TcpInfo {
    mss: u32,
    window_size: u32,
    pmtu: u32,
}

#[get("/")]
async fn get_tcp_info() -> impl Responder {
    let socket = Socket::new(Domain::IPV4, Type::STREAM, None).unwrap();
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    let raw_fd = listener.as_raw_fd();

    let info = unsafe {
        let mut tcp_info: libc::tcp_info = std::mem::zeroed();
        let mut len = std::mem::size_of_val(&tcp_info) as u32;
        libc::getsockopt(
            raw_fd,
            libc::SOL_TCP,
            libc::TCP_INFO,
            &mut tcp_info as *mut _ as *mut _,
            &mut len,
        );
        tcp_info
    };

    let tcp_info = TcpInfo {
        mss: info.tcpi_snd_mss,
        window_size: info.tcpi_snd_wnd,
        pmtu: info.tcpi_pmtu,
    };

    HttpResponse::Ok().json(tcp_info)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(get_tcp_info))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
