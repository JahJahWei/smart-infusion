use amqprs::{callbacks::{DefaultChannelCallback, DefaultConnectionCallback}, channel::{BasicConsumeArguments, BasicPublishArguments, Channel, QueueBindArguments, QueueDeclareArguments}, connection::{Connection, OpenConnectionArguments}, consumer::{AsyncConsumer, DefaultConsumer}, error, BasicProperties};
use anyhow::{Context, Ok, Result};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};
use std::collections::HashMap;
use super::{binding::BindingConsumer, device_data_consumer::DeviceDataConsumer};

static AMQP_MANAGER: OnceCell<Arc<Mutex<AmqpManager>>> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct AmqpConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    exchange: String,
}
impl Default for AmqpConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 5672,
            username: "admin".to_string(),
            password: "120111432@qq.com".to_string(),
            exchange: "amq.topic".to_string(),
        }
    }
}

pub struct AmqpManager {
    connection: Connection,
    channels: HashMap<String, Channel>, 
    config: AmqpConfig,
}

impl AmqpManager {
    async fn new(config: AmqpConfig) -> Result<Self> {
        let connection = Connection::open(&OpenConnectionArguments::new(
                &config.host,
                config.port,
                &config.username,
                &config.password,
            ))
            .await?;

        connection
            .register_callback(DefaultConnectionCallback)
            .await?;
            
        Ok(Self {
            connection,
            channels: HashMap::new(),
            config
        })
    }

    pub async fn register_channel(&mut self, routing_key: &str) -> Result<&Channel> {
        let channel = self.channels
            .get(routing_key).unwrap_or_else(async || {
                let channel = self.connection.open_channel(None).await?;
                channel.register_callback(DefaultChannelCallback).await?;
                self.channels.insert(routing_key.to_string(), channel);
                Ok(channel)
            });


        Ok(channel)
    }

    pub async fn publish(&self, routing_key: &str, content: Vec<u8>) -> Result<()> {
    {
        let channel = self.channels.get(routing_key)?;
        let args = BasicPublishArguments::new(
            &self.config.exchange,
            routing_key,
        );

        channel.basic_publish(BasicProperties::default(), content, args).await?;

        Ok(())
    }

    pub async fn bind_queue(&self, queue_name: &str, routing_key: &str) -> Result<()> {
        let channel: &Channel = self.channels.get(routing_key)?;

        channel.queue_bind(QueueBindArguments::new(
            &queue_name,
            &self.config.exchange,
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
    ) -> Result<()> {
        let channel: &Channel = self.channels.get(routing_key)?;

        let args = BasicConsumeArguments::new(
            queue_name,
            consumer_tag,
        );
        
        channel
            .basic_consume(consumer, args)
            .await?;

        Ok(())
    }

    pub async fn declare_queue(&self, routing_key: &str, queue_name: &str) -> Result<String> {
        let channel: &Channel = self.channels.get(routing_key)?;
        
        let args = QueueDeclareArguments::new(queue_name);

        let (queue_name, _, _) = channel
            .queue_declare(args)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to declare queue: {}", queue_name))?;

        Ok(queue_name)
    }

    pub async fn setup_queue_with_consumer<C: AsyncConsumer + Send + Sync + 'static>(
        &mut self,
        routing_key: &str,
        queue_name: &str,
        consumer_tag: &str,
        consumer: C
    ) -> Result<()> {
        self.register_channel(routing_key).await?;

        let queue_name: String = self.declare_queue(routing_key, queue_name).await?;
        self.bind_queue(&queue_name, routing_key).await?;
        self.setup_consumer(&queue_name, routing_key, consumer_tag, consumer).await?;

        Ok(())
    }
}


pub async fn init_mq() -> Result<()> {
    let config = AmqpConfig::default();
    let mut manager = AmqpManager::new(config).await?;

    manager.setup_queue_with_consumer(
        "device_data",
        "device_data_queue",
        "device_data_consumer_tag",
        DeviceDataConsumer
    )
    .await
    .context("Failed to setup device data queue with consumer")?;

    manager.setup_queue_with_consumer(
        "binding",
        "binding_queue",
        "binding_consumer_tag",
        BindingConsumer
    )
    .await
    .context("Failed to setup binding queue with consumer")?;

    manager.register_channel("alarm").await
        .context("Failed to register alarm channel")?;

    manager.register_channel("controll_device").await
        .context("Failed to register controll_device channel")?;

    let manager = Arc::new(Mutex::new(manager));
    if AMQP_MANAGER.set(manager).is_err() {
        error!("AmqpManager 已经初始化");
    }

    start_listening().await;

    Ok(())
}

pub async fn start_listening() {
    std::future::pending::<()>().await;
}


