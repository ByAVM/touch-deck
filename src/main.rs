use axum::{
    extract::{
        ws::{Message, WebSocket}, ConnectInfo, State, WebSocketUpgrade
    }, response::IntoResponse, routing::get, Router
};
use axum_server::{tls_rustls::RustlsConfig, Handle};
use clap::{arg, Parser};
use rcgen::{CertifiedKey, generate_simple_self_signed};
use tokio::{sync::{mpsc, Mutex}, task};
use vjoy::{ButtonState, VJoy};
use std::{fs::File, io::Write, net::SocketAddr, sync::Arc, time::Duration};

mod types;
mod util;
mod routes;
use types::AppCommand;

async fn handle_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket_handler(socket, addr, state))
}

async fn websocket_handler(mut socket: WebSocket, addr: SocketAddr, state: AppState) {
    println!("Incoming connection from {}", addr);

    while let Some(Ok(Message::Text(msg))) = socket.recv().await {
        println!("Recieved: {}", &msg);

        match serde_json::from_str::<AppCommand>(&msg) {
            Ok(cmd) => {
                if let Err(e) = state.sender.send(cmd).await {
                    eprintln!("Queue send error: {}", e);
                }
            }
            Err(e) => eprintln!("Invalid JSON: {}", e)
        }

        // socket.send(msg).await // Ответ
    }
}

#[derive(Parser)]
struct CliArgs {
    #[arg(long, default_value = "443")]
    port: u16,
}

fn generate_certificates() {
    println!("Generating certificates");

    let subject_alt_names = util::get_device_ips();

    let CertifiedKey { cert, key_pair } = generate_simple_self_signed(subject_alt_names).unwrap();

    let cert_path = util::get_cert_path();
    File::create(cert_path)
    .unwrap()
    .write_all(cert.pem().as_bytes())
    .expect("Could not write certificate to file");

    let key_path = util::get_key_path();
    File::create(key_path)
    .unwrap()
    .write_all(key_pair.serialize_pem().as_bytes())
    .expect("Could not write key_pair to file");
}

async fn init_certificates() -> RustlsConfig {
    let cert_file = util::get_cert_path();
    let key_file = util::get_key_path();

    if !cert_file.exists() || !key_file.exists() {
        generate_certificates();
    }

    // Загружаем существующие сертификаты
    RustlsConfig::from_pem_file(cert_file, key_file)
        .await
        .unwrap()
}

#[derive(Clone)]
struct AppState {
    sender: mpsc::Sender<AppCommand>,
}

async fn shutdown_signal(handle: Handle) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            handle.graceful_shutdown(Some(Duration::from_secs(1)));
        },
        _ = terminate => {
            handle.graceful_shutdown(Some(Duration::from_secs(1)));
        },
    }
}

#[tokio::main]
async fn main() {
    // Сначала тут инициализируется сервер.
    // Если в папке с сертификатом нет сертификатов - они создаются и пользователю предлагают их установить.

    let config = init_certificates().await;

    // Проверяется конфигурация.
    // Далее - инициализируется устройство vJoy.
    let mut vjoy = VJoy::from_default_dll_location().unwrap();
    let mut device = vjoy.get_device_state(1).unwrap();

    // Сброс осей к нейтральному положению
    for axis in device.axes_mut() {
        axis.set((i16::MAX / 2).into());
    }
    let _ = vjoy.update_device_state(&device);

    let shared_vjoy = Arc::new(Mutex::new(vjoy));
    let shared_device = Arc::new(Mutex::new(device));

    let (tx, mut rx) = mpsc::channel::<AppCommand>(100);

    let vjoy_clone = Arc::clone(&shared_vjoy);
    let device_clone = Arc::clone(&shared_device);

    task::spawn(async move {
        while let Some(cmd) = rx.recv().await {

            let mut vjoy = vjoy_clone.lock().await;
            let mut dev = device_clone.lock().await;

            // println!("Axes: {:?}", dev.axes().as_slice());

            // Парсинг команды
            if let Some(axis_cmd) = cmd.axis {
                let axis_id = util::map_axis(axis_cmd.axis.as_str()).unwrap();
                let inval = util::range_to_i32(axis_cmd.value);
                let _ = dev.set_axis(axis_id as u32, inval);
            }

            if let Some(button_cmd) = cmd.button {
                let _ = dev.set_button(button_cmd.button, if button_cmd.pressed { ButtonState::Pressed } else { ButtonState::Released });
            }

            // Применение команды
            let _ = vjoy.update_device_state(&dev);
        }
    });

    let gr_shutdown_handle = Handle::new();
    tokio::spawn(shutdown_signal(gr_shutdown_handle.clone()));

    // Если все хорошо - запускается websocket сервер.

    let app = Router::new()
        .route("/ws", get(handle_ws).with_state(AppState { sender: tx }))
        .route("/check", get(|| async { "Ok" }))
        .route("/get_cert", get(routes::handle_get_cert));

    let args = CliArgs::parse();
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));

    let ips = util::get_device_ips();
    println!("Server enable on ips: {:?}", ips);

    axum_server::bind_rustls(addr, config)
        .handle(gr_shutdown_handle)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap()
}

// При запуске нужно предупреждать пользователя о необходимости установки CA сертификата на клиенте.
