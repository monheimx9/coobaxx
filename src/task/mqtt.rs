use esp_backtrace as _;
use esp_println as _;

use alloc::format;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embedded_nal_async::IpAddr;
use embedded_nal_async::Ipv4Addr;
use embedded_nal_async::SocketAddr;
use embedded_nal_async::TcpConnect;

#[allow(unused)]
use embassy_net::dns::DnsSocket;
#[allow(unused)]
use embedded_nal_async::{AddrType, Dns};
#[allow(unused)]
use smoltcp::wire::DHCP_SERVER_PORT;

use embassy_time::Timer;
use rust_mqtt::{
    client::{
        client::MqttClient,
        client_config::{ClientConfig, MqttVersion},
    },
    packet::v5::publish_packet::QualityOfService,
    utils::rng_generator::CountingRng,
};

use crate::init_board::WifiStack;

const BUFFER_SIZE: usize = 1024;
const SERVER_IP: [u8; 4] = [10, 100, 3, 2];
const SERVER_PORT: u16 = 1883;

#[embassy_executor::task(pool_size = 1)]
pub async fn mqtt_manager(stack: WifiStack) -> ! {
    let mut counter = 0;
    loop {
        if !stack.is_link_up() {
            defmt::info!("Stack link is down");
            Timer::after_secs(5).await;
            continue;
        } else {
            {
                // let host = "broker.emqx.io";
                // let dns_socket = DnsSocket::new(stack);
                // let ip = loop {
                //     if let Ok(ip) = dns_socket.get_host_by_name(host, AddrType::Either).await {
                //         break ip;
                //     }
                //     defmt::info!("Could not resolve hostname");
                //     Timer::after_secs(1).await;
                // };

                let ip4 = IpAddr::from(Ipv4Addr::from(SERVER_IP));
                let state: TcpClientState<3, 4096, 4096> = TcpClientState::new();
                let tcp_client = TcpClient::new(stack, &state);
                defmt::info!("Getting tcp connection");
                let tcp_connection = tcp_client
                    .connect(SocketAddr::new(ip4, SERVER_PORT))
                    .await
                    .unwrap();

                let mut tx_buffer = [0_u8; BUFFER_SIZE];
                let mut rx_buffer = [0_u8; BUFFER_SIZE];
                let mqtt_config: ClientConfig<'_, 3, CountingRng> =
                    ClientConfig::new(MqttVersion::MQTTv5, CountingRng(12334));
                let mut mqtt_client = MqttClient::new(
                    tcp_connection,
                    &mut tx_buffer,
                    BUFFER_SIZE,
                    &mut rx_buffer,
                    BUFFER_SIZE,
                    mqtt_config,
                );
                defmt::info!("Attempting broker connection");
                mqtt_client.connect_to_broker().await.unwrap();
                // mqtt_client.subscribe_to_topic("orsoporc/1").await.unwrap();
                defmt::info!("Connected to broker");
                mqtt_client
                    .send_message(
                        "orsoporc/",
                        format!("Megaporc N° {}", counter).as_bytes(),
                        QualityOfService::QoS0,
                        false,
                    )
                    .await
                    .unwrap();
                defmt::info!("MQTT Message sent: Megaporc N°{} completed", counter);

                Timer::after_secs(20).await;
            }

            counter += 1;
        }
    }
}
