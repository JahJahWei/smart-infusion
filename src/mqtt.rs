use rumqttc::{AsyncClient, MqttOptions, QoS, Event, Packet};
use std::time::Duration;
use tokio::sync::{mpsc, Mutex as TokioMutex, RwLock};
use std::sync::Arc;
use tokio::task;
use std::fmt;
use std::error::Error;
use once_cell::sync::Lazy;
use std::collections::HashMap;

// 创建自定义错误类型
#[derive(Debug)]
pub enum MqttError {
    NotConnected,
    ClientError(rumqttc::ClientError),
}

impl fmt::Display for MqttError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MqttError::NotConnected => write!(f, "MQTT client not connected"),
            MqttError::ClientError(e) => write!(f, "MQTT client error: {}", e),
        }
    }
}

impl Error for MqttError {}

impl From<rumqttc::ClientError> for MqttError {
    fn from(error: rumqttc::ClientError) -> Self {
        MqttError::ClientError(error)
    }
}

// 使用tokio的Mutex代替std::sync::Mutex
static MQTT_CLIENT: Lazy<Arc<TokioMutex<Option<AsyncClient>>>> = Lazy::new(|| {
    Arc::new(TokioMutex::new(None))
});

// 定义消息处理器类型
type MessageHandler = Box<dyn Fn(&str, &str) -> () + Send + Sync>;

// 全局消息处理器映射表
static TOPIC_HANDLERS: Lazy<RwLock<HashMap<String, MessageHandler>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

pub async fn init_mqtt(host: String, port: u16, client_id: String) {
    let mut mqtt_options = MqttOptions::new(client_id, host, port);
    mqtt_options.set_keep_alive(Duration::from_secs(30));
    
    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
    
    // 保存客户端实例
    let client_clone = client.clone();
    *MQTT_CLIENT.lock().await = Some(client_clone);
    
    task::spawn(async move {
        println!("MQTT事件循环已启动");
        while let Ok(event) = eventloop.poll().await {
            match event {
                Event::Incoming(Packet::Publish(publish)) => {
                    let topic = publish.topic;
                    let payload = String::from_utf8_lossy(&publish.payload);
                    println!("收到MQTT消息，主题: {}, 内容: {}", topic, payload);
                    
                    // 查找并执行对应的处理器
                    if let Some(handlers) = TOPIC_HANDLERS.read().await.get(&topic) {
                        handlers(&topic, &payload);
                    } else {
                        // 使用通配符匹配
                        let handlers = TOPIC_HANDLERS.read().await;
                        for (pattern, handler) in handlers.iter() {
                            if topic_matches(pattern, &topic) {
                                handler(&topic, &payload);
                                break;
                            }
                        }
                    }
                },
                Event::Incoming(Packet::ConnAck(_)) => {
                    println!("MQTT连接成功");
                },
                Event::Incoming(Packet::PingResp) => {
                    println!("收到心跳响应");
                },
                Event::Incoming(Packet::Disconnect) => {
                    println!("连接断开，尝试重连");
                    break;
                },
                _ => {}
            }
        }
        println!("MQTT事件循环已结束");
    });
}

// 发送MQTT消息的函数
pub async fn publish_message(topic: &str, payload: &str) -> Result<(), MqttError> {
    let client_guard = MQTT_CLIENT.lock().await;
    
    if let Some(client) = &*client_guard {
        println!("发送MQTT消息，主题: {}, 内容: {}", topic, payload);
        return client.publish(topic, QoS::AtLeastOnce, false, payload).await.map_err(|e| e.into());
    }
    
    // 使用自定义错误类型
    Err(MqttError::NotConnected)
}

// 获取MQTT客户端实例是否已初始化
pub async fn is_mqtt_connected() -> bool {
    MQTT_CLIENT.lock().await.is_some()
}

// 注册消息处理器
pub async fn register_handler(topic: &str, handler: impl Fn(&str, &str) -> () + Send + Sync + 'static) {
    let mut handlers = TOPIC_HANDLERS.write().await;
    handlers.insert(topic.to_string(), Box::new(handler));
}

// 辅助函数：检查主题是否匹配模式
fn topic_matches(pattern: &str, topic: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let topic_parts: Vec<&str> = topic.split('/').collect();
    
    if pattern_parts.len() != topic_parts.len() && !pattern.contains("+") && !pattern.contains("#") {
        return false;
    }
    
    for (p, t) in pattern_parts.iter().zip(topic_parts.iter()) {
        match *p {
            "+" => continue,
            "#" => return true,
            _ if *p != *t => return false,
            _ => continue,
        }
    }
    
    true
} 