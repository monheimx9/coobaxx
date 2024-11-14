use esp_backtrace as _;
use esp_println as _;

use alloc::format;
use embassy_net::{
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
    Stack,
};
use embassy_time::Timer;
use embedded_nal_async::{AddrType, Dns, IpAddr, Ipv4Addr, SocketAddr, TcpConnect};
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};
use rust_mqtt::{
    client::{
        client::MqttClient,
        client_config::{ClientConfig, MqttVersion},
    },
    packet::v5::publish_packet::QualityOfService,
    utils::rng_generator::CountingRng,
};

pub async fn send_mqtt_message(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) -> ! {
    let mut counter = 0;
    loop {
        if !stack.is_link_up() {
            defmt::info!("Stack link is down");
            Timer::after_secs(5).await;
            continue;
        } else {
            {
                let host = "broker.emqx.io";
                let dns_socket = DnsSocket::new(stack);
                let ip = loop {
                    if let Ok(ip) = dns_socket.get_host_by_name(host, AddrType::Either).await {
                        break ip;
                    }
                    defmt::info!("Could not resolve hostname");
                    Timer::after_secs(1).await;
                };

                let ip4 = Ipv4Addr::new(10, 100, 3, 2);
                let ip4 = IpAddr::from(ip4);

                let state: TcpClientState<3, 4096, 4096> = TcpClientState::new();
                let tcp_client = TcpClient::new(stack, &state);
                defmt::info!("Getting tcp connection");
                let tcp_connection = tcp_client
                    .connect(SocketAddr::new(ip4, 1883))
                    .await
                    .unwrap();

                let mut tx_buffer = [0_u8; 1024];
                let mut rx_buffer = [0_u8; 1024];
                let mqtt_config: ClientConfig<'_, 3, CountingRng> =
                    ClientConfig::new(MqttVersion::MQTTv5, CountingRng(12334));
                let mut mqtt_client = MqttClient::new(
                    tcp_connection,
                    &mut tx_buffer,
                    1024,
                    &mut rx_buffer,
                    1024,
                    mqtt_config,
                );
                defmt::info!("Attempting broker connection");
                mqtt_client.connect_to_broker().await.unwrap();
                defmt::info!("Connected to broker");
                mqtt_client
                    .send_message(
                        "orsoporc",
                        format!("Megaporc N° {}", counter).as_bytes(),
                        QualityOfService::QoS1,
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
