use core::str::from_utf8;
use embassy_futures::select::select3;
use embassy_futures::select::Either3;
use heapless::String;
use heapless::Vec;

use defmt::{debug, error, info, warn};
use esp_backtrace as _;

use embassy_net::tcp::client::TcpConnection;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_net::tcp::TcpSocket;
use embassy_net::IpAddress;
use embassy_net::Ipv4Address;
use embedded_nal_async::IpAddr;
use embedded_nal_async::Ipv4Addr;
use embedded_nal_async::SocketAddr;
use embedded_nal_async::TcpConnect;

#[allow(unused)]
use embassy_net::dns::DnsSocket;

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

use crate::constant::STRING_SIZE;
use crate::constant::{MQTT_SUB_TOPICS, MQTT_SUB_TOPICS_SIZE};
use crate::task::state::MqttMessage;
use crate::task::state::CURRENT_DEVICE_NAME;
use crate::task::task_messages::Events;
use crate::task::task_messages::EVENT_CHANNEL;
use crate::task::task_messages::MQTT_SIGNAL_BROKER_PING;
use crate::task::task_messages::MQTT_SIGNAL_SEND;

const POOL_TXRX_SZ: usize = 256;
const BUFFER_SIZE: usize = 128;
const SERVER_IP: [u8; 4] = [10, 100, 3, 2];
const SERVER_PORT: u16 = 1883;

#[embassy_executor::task(pool_size = 1)]
pub async fn mqtt_manager(stack: WifiStack) -> ! {
    loop {
        'mqttsend: {
            if !stack.is_link_up() {
                defmt::info!("Stack link is down");
                Timer::after_secs(5).await;
                break 'mqttsend;
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

                    //TCP Connection

                    let state: TcpClientState<3, POOL_TXRX_SZ, POOL_TXRX_SZ> =
                        TcpClientState::new();
                    let tcp_client = TcpClient::new(stack, &state);

                    // let (mut tx_buff, mut rx_buff) = ([0_u8; 4096], [0_u8; 4096]);
                    // let mut sock = TcpSocket::new(stack, &mut rx_buff, &mut tx_buff);
                    // let endpoint = (Ipv4Address::from_bytes(&SERVER_IP), SERVER_PORT);
                    // sock.connect(endpoint).await.unwrap();

                    let ip4 = IpAddress::from(Ipv4Address::from_bytes(&SERVER_IP));

                    let ip4 = IpAddr::from(Ipv4Addr::from(SERVER_IP));
                    defmt::info!("Getting tcp connection");
                    let tcp_connection = tcp_client
                        .connect(SocketAddr::new(ip4, SERVER_PORT))
                        .await
                        .unwrap();
                    let mut tx_buffer = [0; BUFFER_SIZE];
                    let mut rx_buffer = [0; BUFFER_SIZE];

                    //Client config
                    let mut death_payload: String<STRING_SIZE> = String::new();
                    death_payload.push_str("dead=").unwrap();
                    death_payload.push_str(CURRENT_DEVICE_NAME).unwrap();
                    let mut mqtt_config: ClientConfig<'_, 3, CountingRng> =
                        ClientConfig::new(MqttVersion::MQTTv5, CountingRng(12334));
                    mqtt_config.add_will("magneporc/devices/ping", death_payload.as_bytes(), false);
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
                    defmt::info!("Connected to broker");
                    {
                        let topics: Vec<&str, MQTT_SUB_TOPICS_SIZE> =
                            Vec::from_slice(MQTT_SUB_TOPICS).unwrap();
                        mqtt_client.subscribe_to_topics(&topics).await.unwrap();
                    }

                    'mqtt_action: loop {
                        match select3(
                            mqtt_client.receive_message(),
                            MQTT_SIGNAL_SEND.wait(),
                            MQTT_SIGNAL_BROKER_PING.wait(),
                        )
                        .await
                        {
                            Either3::First(msg) => {
                                if let Ok((topic, payload)) = msg {
                                    defmt::info!("Message received from topic: {}", topic);
                                    let pa = from_utf8(payload).unwrap();
                                    defmt::info!("Content:{}", pa);
                                    if let Some(mqtt_msg) = MqttMessage::try_new(topic, payload) {
                                        EVENT_CHANNEL.send(Events::MessageReceived(mqtt_msg)).await;
                                    };
                                } else {
                                    warn!("Network error occured");
                                    break 'mqtt_action;
                                }
                            }
                            Either3::Second(cmd) => {
                                if mqtt_client
                                    .send_message(
                                        cmd.topic(),
                                        cmd.payload().as_bytes(),
                                        QualityOfService::QoS0,
                                        false,
                                    )
                                    .await
                                    .is_ok()
                                {
                                    defmt::info!("MQTT Message sent: {}", cmd.payload().as_str());
                                } else {
                                    warn!("Network error occured");
                                    break 'mqtt_action;
                                }
                            }
                            Either3::Third(_) => {
                                if mqtt_client.send_ping().await.is_ok() {
                                    debug!("PING sent to Broker")
                                } else {
                                    warn!("Couln't ping the Broker")
                                };
                            }
                        }
                    }
                }
            }
        }
    }
}
