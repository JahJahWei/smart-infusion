use rumqttd::*;
use uuid::Uuid;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicUsize, Ordering};

// 因为Broker没有实现Clone和Debug，所以简化设计，只保存一个标志表示服务器已启动
static MQTT_STARTED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

// 添加消息计数器
static MESSAGE_COUNT: AtomicUsize = AtomicUsize::new(0);

// MQTT配置结构
#[derive(Debug, Serialize, Deserialize)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub max_client_id_len: usize,
}

impl Default for MqttConfig {
    fn default() -> Self {
        MqttConfig {
            host: "0.0.0.0".to_string(),
            port: 1883,
            max_connections: 1000,
            max_client_id_len: 256,
        }
    }
}

pub fn start_mqtt_broker(config: MqttConfig) {
    let mut started = MQTT_STARTED.lock().unwrap();
    if *started {
        println!("MQTT服务器已经在运行");
        return;
    }
    
    println!("开始启动MQTT服务器...");
    
    let mut router_config = RouterConfig::default();
    router_config.max_connections = config.max_connections;
    router_config.max_segment_size = 1024 * 1024 * 1024;
    router_config.max_segment_count = 1000;
    router_config.max_outgoing_packet_count = 1000;

    let server_settings = ServerSettings {
        name: "mqtt_server".to_string(),
        listen: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), config.port),
        tls: None,
        next_connection_delay_ms: 1000,
        connections: ConnectionSettings {
            connection_timeout_ms: 10000,
            max_payload_size: 2 * 1024 * 1024,
            max_inflight_count: 100,
            auth: None,
            external_auth: None,
            dynamic_filters: false,
        },
    };

    let mut v3_protocol = HashMap::new();
    v3_protocol.insert("server_settings".to_string(), server_settings.clone());
    
    let mut v5_protocol = HashMap::new();
    v5_protocol.insert("server_settings".to_string(), server_settings);

    let mut broker_config = Config::default();
    broker_config.id = 332;
    broker_config.router = router_config;
    broker_config.v4 = Some(v3_protocol);
    broker_config.v5 = Some(v5_protocol);

    let mut broker = Broker::new(broker_config);
    
    // 添加消息处理器
    let router = broker.router();
    
    thread::spawn(move || {
        println!("MQTT服务器线程启动");
        
        // 在单独的线程中监控消息
        thread::spawn(move || {
            println!("消息监控线程启动");
            loop {
                if let Ok(notification) = router.recv() {
                    match notification {
                        Notification::(connection) => {
                            println!("新的连接: id={}", connection.id);
                        },
                        Notification::Message(publish) => {
                            MESSAGE_COUNT.fetch_add(1, Ordering::SeqCst);
                            println!("收到消息: topic={}, qos={}, size={}bytes", 
                                publish.topic,
                                publish.qos,
                                publish.payload.len()
                            );
                            
                            // 如果需要，可以打印消息内容
                            if let Ok(payload) = String::from_utf8(publish.payload.clone()) {
                                println!("消息内容: {}", payload);
                            }
                        },
                        Notification::Disconnect(connection_id, _) => {
                            println!("连接断开: id={}", connection_id);
                        },
                        _ => {}
                    }
                }
            }
        });

        if let Err(e) = broker.start() {
            eprintln!("MQTT服务器启动失败: {}", e);
        }
    });
    
    thread::sleep(Duration::from_millis(100));
    
    *started = true;
    println!("MQTT服务器已启动在 {}:{}", config.host, config.port);
}

// 添加获取消息统计的函数
pub fn get_message_count() -> usize {
    MESSAGE_COUNT.load(Ordering::SeqCst)
}

// 添加重置消息计数的函数
pub fn reset_message_count() {
    MESSAGE_COUNT.store(0, Ordering::SeqCst);
}
