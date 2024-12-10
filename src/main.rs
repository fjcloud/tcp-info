use actix_web::{get, head, web::Data, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use socket2::{Socket, Type, Protocol, Domain};
use std::os::fd::AsRawFd;

#[derive(Serialize, Debug)]
struct TcpConnectionInfo {
    state: String,
    ca_state: String,
    retransmits: u8,
    probes: u8,
    backoff: u8,
    rto: String,
    ato: String,
    snd_mss: u32,
    rcv_mss: u32,
    unacked: u32,
    sacked: u32,
    lost: u32,
    retrans: u32,
    fackets: u32,
    last_data_sent: String,
    last_ack_sent: String,
    last_data_recv: String,
    last_ack_recv: String,
    pmtu: u32,
    rcv_ssthresh: u32,
    rtt: String,
    rttvar: String,
    snd_ssthresh: u32,
    snd_cwnd: u32,
    advmss: u32,
    reordering: u32,
    rcv_rtt: String,
    rcv_space: u32,
    total_retrans: u32,
    local_addr: String,
    peer_addr: Option<String>,
    tcp_nodelay: bool,
    keepalive: bool
}

struct TcpSocket {
    socket: Socket,
}

impl TcpSocket {
    fn new(socket: Socket) -> Self {
        Self { socket }
    }

    fn get_tcp_info(&self, req: &HttpRequest) -> Option<TcpConnectionInfo> {
        unsafe {
            let mut tcp_info: libc::tcp_info = std::mem::zeroed();
            let mut len = std::mem::size_of_val(&tcp_info) as u32;
            
            if libc::getsockopt(
                self.socket.as_raw_fd(),
                libc::SOL_TCP,
                libc::TCP_INFO,
                &mut tcp_info as *mut _ as *mut _,
                &mut len
            ) == 0 {
                Some(TcpConnectionInfo {
                    state: tcp_state_to_string(tcp_info.tcpi_state),
                    ca_state: ca_state_to_string(tcp_info.tcpi_ca_state),
                    retransmits: tcp_info.tcpi_retransmits,
                    probes: tcp_info.tcpi_probes,
                    backoff: tcp_info.tcpi_backoff,
                    rto: microseconds_to_human(tcp_info.tcpi_rto),
                    ato: microseconds_to_human(tcp_info.tcpi_ato),
                    snd_mss: tcp_info.tcpi_snd_mss,
                    rcv_mss: tcp_info.tcpi_rcv_mss,
                    unacked: tcp_info.tcpi_unacked,
                    sacked: tcp_info.tcpi_sacked,
                    lost: tcp_info.tcpi_lost,
                    retrans: tcp_info.tcpi_retrans,
                    fackets: tcp_info.tcpi_fackets,
                    last_data_sent: microseconds_to_human(tcp_info.tcpi_last_data_sent),
                    last_ack_sent: microseconds_to_human(tcp_info.tcpi_last_ack_sent),
                    last_data_recv: microseconds_to_human(tcp_info.tcpi_last_data_recv),
                    last_ack_recv: microseconds_to_human(tcp_info.tcpi_last_ack_recv),
                    pmtu: tcp_info.tcpi_pmtu,
                    rcv_ssthresh: tcp_info.tcpi_rcv_ssthresh,
                    rtt: microseconds_to_human(tcp_info.tcpi_rtt),
                    rttvar: microseconds_to_human(tcp_info.tcpi_rttvar),
                    snd_ssthresh: tcp_info.tcpi_snd_ssthresh,
                    snd_cwnd: tcp_info.tcpi_snd_cwnd,
                    advmss: tcp_info.tcpi_advmss,
                    reordering: tcp_info.tcpi_reordering,
                    rcv_rtt: microseconds_to_human(tcp_info.tcpi_rcv_rtt),
                    rcv_space: tcp_info.tcpi_rcv_space,
                    total_retrans: tcp_info.tcpi_total_retrans,
                    local_addr: req.connection_info().host().to_string(),
                    peer_addr: req.peer_addr().map(|addr| addr.to_string()),
                    tcp_nodelay: self.socket.nodelay().unwrap_or(false),
                    keepalive: self.socket.keepalive().unwrap_or(false)
                })
            } else {
                None
            }
        }
    }
}

fn tcp_state_to_string(state: u8) -> String {
    String::from(match state {
        1 => "ESTABLISHED",
        2 => "SYN_SENT",
        3 => "SYN_RECV",
        4 => "FIN_WAIT1",
        5 => "FIN_WAIT2",
        6 => "TIME_WAIT",
        7 => "CLOSE",
        8 => "CLOSE_WAIT",
        9 => "LAST_ACK",
        10 => "LISTEN",
        11 => "CLOSING",
        _ => "UNKNOWN"
    })
}

fn ca_state_to_string(state: u8) -> String {
    String::from(match state {
        0 => "OPEN",
        1 => "DISORDER",
        2 => "CWR",
        3 => "RECOVERY",
        4 => "LOSS",
        _ => "UNKNOWN"
    })
}

fn microseconds_to_human(us: u32) -> String {
    match us {
        0 => "0".to_string(),
        1..=999 => format!("{}µs", us),
        1000..=999999 => format!("{}ms", us/1000),
        _ => format!("{}s", us/1000000)
    }
}

#[get("/")]
async fn get_tcp_info(req: HttpRequest, socket: Data<TcpSocket>) -> impl Responder {
    match socket.get_tcp_info(&req) {
        Some(info) => HttpResponse::Ok().json(info),
        None => HttpResponse::InternalServerError().body("Failed to get TCP info")
    }
}

#[head("/")]
async fn head_tcp_info(req: HttpRequest, socket: Data<TcpSocket>) -> impl Responder {
    match socket.get_tcp_info(&req) {
        Some(info) => HttpResponse::Ok().json(info),
        None => HttpResponse::InternalServerError().body("Failed to get TCP info")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_reuse_address(true)?;
    socket.set_nonblocking(true)?;
    socket.set_nodelay(true)?;

    let tcp_socket = Data::new(TcpSocket::new(socket));
    println!("Server listening on 0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::clone(&tcp_socket))
            .service(get_tcp_info)
            .service(head_tcp_info)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
