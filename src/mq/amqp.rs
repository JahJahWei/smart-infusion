use amqprs::{callbacks::{DefaultChannelCallback, DefaultConnectionCallback}, channel::{BasicConsumeArguments, BasicPublishArguments, Channel, QueueBindArguments, QueueDeclareArguments}, connection::{Connection, OpenConnectionArguments}, consumer::{AsyncConsumer, DefaultConsumer}, BasicProperties};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};
use std::collections::HashMap;

use super::{binding::BindingConsumer, device_data_consumer::DeviceDataConsumer, device_status::DeviceStatusConsumer};

static AMQP_MANAGER: OnceCell<Arc<Mutex<AmqpManager>>> = OnceCell::new();

pub struct AmqpManager {
    connection: Connection,
    channels: HashMap<String, Channel>, 
}

impl AmqpManager {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let connection = Connection::open(&OpenConnectionArguments::new(
            "127.0.0.1",
            5672,
            "admin",
            "120111432@qq.com",
        ))
        .await?;

        connection
            .register_callback(DefaultConnectionCallback)
            .await?;
            
        Ok(Self {
            connection,
            channels: HashMap::new(),
        })
    }

    pub async fn register_channel(&mut self, routing_key: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.channels.contains_key(routing_key) {
            return Ok(());
        }

        let channel = match self.connection.open_channel(None).await {
            Ok(channel) => channel,
            Err(e) => {
                eprintln!("Failed to open channel: {}", e);
                return Err(e.into());
            }
        };
        channel
            .register_callback(DefaultChannelCallback)
            .await?;
        
        self.channels.insert(routing_key.to_string(), channel);
        Ok(())
    }

    pub fn get_channel(&self, routing_key: &str) -> Option<&Channel> {
        self.channels.get(routing_key)
    }

    pub async fn publish(&self, exchange: &str, routing_key: &str, content: Vec<u8>) 
        -> Result<(), Box<dyn std::error::Error>> 
    {
        let channel = self.channels.get(routing_key)
            .ok_or(format!("未找到routing_key '{}'对应的通道", routing_key))?;
            
        let args = BasicPublishArguments::new(
            exchange,
            routing_key,
        );

        channel.basic_publish(BasicProperties::default(), content, args).await?;

        Ok(())
    }

    pub async fn bind_queue(&self, queue_name: &str, routing_key: &str, exchange_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let channel = self.channels.get(routing_key)
            .ok_or(format!("未找到routing_key '{}'对应的通道", routing_key))?;

        channel.queue_bind(QueueBindArguments::new(
            &queue_name,
            exchange_name,
            routing_key,
        ))
        .await?;

        Ok(())
    }

    pub async fn setup_consumer<C: AsyncConsumer + Send + Sync + 'static>(
        &self, 
        queue_name: &str,
        routing_key: &str, 
        consumer_tag: &str,
        consumer: C
    ) -> Result<(), Box<dyn std::error::Error>> {
        let channel = self.channels.get(routing_key)
            .ok_or(format!("未找到routing_key '{}'对应的通道", routing_key))?;

        let args = BasicConsumeArguments::new(
            queue_name,
            consumer_tag,
        );
        
        channel
            .basic_consume(consumer, args)
            .await?;

        Ok(())
    }

    pub async fn declare_queue(&self, routing_key: &str, queue_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let channel = self.channels.get(routing_key)
            .ok_or(format!("未找到routing_key '{}'对应的通道", routing_key))?;
        
        let args = QueueDeclareArguments::new(
            queue_name,
        );

        let queue_name = match channel
            .queue_declare(args)
            .await {
                Ok(result) => {
                    match result {
                        Some((queue_name, _, _)) => queue_name,
                        None => {
                            eprintln!("Failed to declare queue");
                            return Err("Failed to declare queue".into());
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Failed to declare queue: {}", e);
                    return Err(e.into());
                }
            };
            
        Ok(queue_name)
    }
}


pub async fn init_mq() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = match AmqpManager::new().await {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("Failed to create AmqpManager: {}", e);
            return Err(e);
        }
    };
    
    let exchange_name = "amq.topic";
    
    let device_data_routing_key = "device_data";
    match manager.register_channel(device_data_routing_key).await {
        Ok(_) => println!("device data channel is registered"),
        Err(e) => {
            eprintln!("Failed to register device data channel: {}", e);
            return Err(e);
        }
    }
    let device_data_queue = match manager.declare_queue(device_data_routing_key, "device_data_queue").await {
        Ok(queue) => {
            println!("device_data_queue declared: {}", queue);
            queue
        },
        Err(e) => {
            eprintln!("Failed to declare device_status_queue: {}", e);
            return Err(e);
        }
    };
    match manager.bind_queue(&device_data_queue, device_data_routing_key, exchange_name).await {
        Ok(_) => println!("device_data_queue bound"),
        Err(e) => {
            eprintln!("Failed to bind device_data_queue: {}", e);
            return Err(e);
        }
    }
    match manager.setup_consumer(
        &device_data_queue,
        device_data_routing_key,
        "device_data_consumer_tag",
        DeviceDataConsumer
    ).await {
        Ok(_) => println!("device_data_consumer_tag setup"),
        Err(e) => {
            eprintln!("Failed to setup device_status_consumer_tag: {}", e);
            return Err(e);
        }
    };

    //binding queue
    let binding_routing_key = "binding";
    let binding_queue_name = "binding_queue";
    match manager.register_channel(binding_routing_key).await {
        Ok(_) => println!("binding_routing_key registered"),
        Err(e) => {
            eprintln!("Failed to register binding_routing_key: {}", e);
            return Err(e);
        }
    }
    let binding_queue = match manager.declare_queue(binding_routing_key, binding_queue_name).await {
        Ok(queue) => {
            println!("binding_queue declared: {}", queue);
            queue
        },
        Err(e) => {
            eprintln!("Failed to declare binding_queue: {}", e);
            return Err(e);
        }
    };
    match manager.bind_queue(&binding_queue, binding_routing_key, exchange_name).await {
        Ok(_) => println!("binding_queue bound"),
        Err(e) => {
            eprintln!("Failed to bind binding_queue: {}", e);
            return Err(e);
        }
    }
    match manager.setup_consumer(
        &binding_queue,
        binding_routing_key,
        "binding_consumer_tag",
        BindingConsumer
    ).await {
        Ok(_) => println!("binding_consumer_tag setup"),
        Err(e) => {
            eprintln!("Failed to setup binding_consumer_tag: {}", e);
            return Err(e);
        }
    };

    //drip rate queue
    let drip_rate_routing_key = "drip_rate";
    let drip_rate_queue_name = "drip_rate_queue";
    match manager.register_channel(drip_rate_routing_key).await {
        Ok(_) => println!("drip_rate_routing_key registered"),
        Err(e) => {
            eprintln!("Failed to register drip_rate_routing_key: {}", e);
            return Err(e);
        }
    }
    let drip_rate_queue = match manager.declare_queue(drip_rate_routing_key, drip_rate_queue_name).await {
        Ok(queue) => {
            println!("drip_rate_queue declared: {}", queue);
            queue
        },
        Err(e) => {
            eprintln!("Failed to declare drip_rate_queue: {}", e);
            return Err(e);
        }
    };
    match manager.bind_queue(&drip_rate_queue, drip_rate_routing_key, exchange_name).await {
        Ok(_) => println!("drip_rate_queue bound"),
        Err(e) => {
            eprintln!("Failed to bind drip_rate_queue: {}", e);
            return Err(e);
        }
    }

    let alarm_routing_key = "alarm";
    let alarm_queue_name = "alarm_queue";
    match manager.register_channel(alarm_routing_key).await {
        Ok(_) => println!("alarm_routing_key registered"),
        Err(e) => {
            eprintln!("Failed to register alarm_routing_key: {}", e);
            return Err(e);
        }
    }
    let alarm_queue = match manager.declare_queue(alarm_routing_key, alarm_queue_name).await {
        Ok(queue) => {
            println!("alarm_queue declared: {}", queue);
            queue
        },
        Err(e) => {
            eprintln!("Failed to declare alarm_queue: {}", e);
            return Err(e);
        }
    };
    match manager.bind_queue(&alarm_queue, alarm_routing_key, exchange_name).await {
        Ok(_) => println!("alarm_queue bound"),
        Err(e) => {
            eprintln!("Failed to bind alarm_queue: {}", e);
            return Err(e);
        }
    }

    let manager = Arc::new(Mutex::new(manager));
    if AMQP_MANAGER.set(manager).is_err() {
        error!("AmqpManager 已经初始化");
    }

    start_listening().await;

    Ok(())
}

pub fn get_amqp_manager() -> Option<Arc<Mutex<AmqpManager>>> {
    AMQP_MANAGER.get().cloned()
}

pub async fn start_listening() {
    std::future::pending::<()>().await;
}


