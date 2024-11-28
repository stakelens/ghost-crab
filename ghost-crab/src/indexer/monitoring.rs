use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug)]
pub struct HandlerMetrics {
    name: String,
    handler_type: HandlerType,
    active_tasks: AtomicU64,
    failed_tasks: AtomicU64,
    processed_tasks: AtomicU64,
    last_error: Arc<RwLock<Option<(Instant, String)>>>,
    last_active: Arc<RwLock<Instant>>,
    last_processed_block: AtomicU64,
}

#[derive(Debug, Clone, Serialize)]
pub enum HandlerType {
    Event,
    Block,
    Template,
}

#[derive(Debug, Serialize)]
pub struct HandlerStatus {
    name: String,
    handler_type: HandlerType,
    active_tasks: u64,
    failed_tasks: u64,
    processed_tasks: u64,
    last_error: Option<(Duration, String)>,
    idle_duration: Duration,
    last_processed_block: u64,
    health_status: HealthStatus,
}

#[derive(Debug, Clone, Serialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Failed,
}

#[derive(Debug, Serialize)]
struct MonitoringLog {
    #[serde(with = "chrono::serde::ts_seconds")]
    timestamp: DateTime<Utc>,
    handler_name: String,
    handler_type: HandlerType,
    status: String,
    last_block: u64,
    #[serde(with = "humantime_serde")]
    idle_duration: Duration,
    last_error: Option<String>,
}

impl HandlerMetrics {
    pub fn new(name: String, handler_type: HandlerType) -> Arc<Self> {
        Arc::new(Self {
            name,
            handler_type,
            active_tasks: AtomicU64::new(0),
            failed_tasks: AtomicU64::new(0),
            processed_tasks: AtomicU64::new(0),
            last_error: Arc::new(RwLock::new(None)),
            last_active: Arc::new(RwLock::new(Instant::now())),
            last_processed_block: AtomicU64::new(0),
        })
    }

    pub fn task_started(&self) {
        self.active_tasks.fetch_add(1, Ordering::SeqCst);
        tokio::spawn({
            let last_active = self.last_active.clone();
            async move {
                *last_active.write().await = Instant::now();
            }
        });
    }

    pub fn task_completed(&self, block_number: u64) {
        self.active_tasks.fetch_sub(1, Ordering::SeqCst);
        self.processed_tasks.fetch_add(1, Ordering::SeqCst);
        self.last_processed_block.fetch_max(block_number, Ordering::SeqCst);
    }

    pub fn task_failed(&self, error: String) {
        self.active_tasks.fetch_sub(1, Ordering::SeqCst);
        self.failed_tasks.fetch_add(1, Ordering::SeqCst);
        tokio::spawn({
            let last_error = self.last_error.clone();
            async move {
                *last_error.write().await = Some((Instant::now(), error));
            }
        });
    }

    pub async fn get_status(&self) -> HandlerStatus {
        let last_error = self.last_error.read().await;
        let last_active = self.last_active.read().await;

        let idle_duration = last_active.elapsed();
        let health_status = self.determine_health(idle_duration).await;

        HandlerStatus {
            name: self.name.clone(),
            handler_type: self.handler_type.clone(),
            active_tasks: self.active_tasks.load(Ordering::SeqCst),
            failed_tasks: self.failed_tasks.load(Ordering::SeqCst),
            processed_tasks: self.processed_tasks.load(Ordering::SeqCst),
            last_error: last_error.as_ref().map(|(instant, msg)| (instant.elapsed(), msg.clone())),
            idle_duration,
            last_processed_block: self.last_processed_block.load(Ordering::SeqCst),
            health_status,
        }
    }

    async fn determine_health(&self, idle_duration: Duration) -> HealthStatus {
        const WARNING_THRESHOLD: Duration = Duration::from_secs(300); // 5 minutes
        const FAILURE_THRESHOLD: Duration = Duration::from_secs(900); // 15 minutes

        let has_recent_error = self
            .last_error
            .read()
            .await
            .as_ref()
            .map(|(instant, _)| instant.elapsed() < Duration::from_secs(3600))
            .unwrap_or(false);

        match (idle_duration, has_recent_error) {
            (idle, _) if idle > FAILURE_THRESHOLD => HealthStatus::Failed,
            (idle, true) if idle > WARNING_THRESHOLD => HealthStatus::Warning,
            (idle, false) if idle > WARNING_THRESHOLD => HealthStatus::Warning,
            _ => HealthStatus::Healthy,
        }
    }

    async fn log_status(&self, status: &HandlerStatus) {
        let log = MonitoringLog {
            timestamp: Utc::now(),
            handler_name: status.name.clone(),
            handler_type: status.handler_type.clone(),
            status: format!("{:?}", status.health_status),
            last_block: status.last_processed_block,
            idle_duration: status.idle_duration,
            last_error: status.last_error.as_ref().map(|(_, msg)| msg.clone()),
        };

        match &status.health_status {
            HealthStatus::Healthy => {
                info!(
                    handler = %log.handler_name,
                    type = ?log.handler_type,
                    last_block = log.last_block,
                    idle_for = ?log.idle_duration,
                    "Handler operating normally"
                );
            }
            HealthStatus::Warning => {
                warn!(
                    handler = %log.handler_name,
                    type = ?log.handler_type,
                    last_block = log.last_block,
                    idle_for = ?log.idle_duration,
                    error = log.last_error.as_deref().unwrap_or("No error recorded"),
                    "Handler entering warning state"
                );
            }
            HealthStatus::Failed => {
                error!(
                    handler = %log.handler_name,
                    type = ?log.handler_type,
                    last_block = log.last_block,
                    idle_for = ?log.idle_duration,
                    error = log.last_error.as_deref().unwrap_or("No error recorded"),
                    "Handler has failed"
                );
            }
        }
    }
}

pub struct MonitoringSystem {
    handlers: Arc<RwLock<Vec<Arc<HandlerMetrics>>>>,
}

impl MonitoringSystem {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { handlers: Arc::new(RwLock::new(Vec::new())) })
    }

    pub async fn start_logging(self: Arc<Self>) {
        loop {
            if let Err(e) = self.log_status_safely().await {
                eprintln!("Error in monitoring system: {}. Monitoring will continue.", e);
            }

            tokio::time::sleep(Duration::from_secs(15)).await;
        }
    }

    async fn log_status_safely(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let statuses = self.get_all_statuses().await;

        for status in statuses {
            match self.log_handler_status(&status).await {
                Ok(_) => continue,
                Err(e) => eprintln!(
                    "Failed to log status for handler {}: {}. Continuing with other handlers.",
                    status.name, e
                ),
            }
        }
        Ok(())
    }

    async fn log_handler_status(
        &self,
        status: &HandlerStatus,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let handlers = self.handlers.read().await;

        if let Some(handler) = handlers.iter().find(|h| h.name == status.name) {
            handler.log_status(status).await;
        }
        Ok(())
    }

    pub async fn register_handler(
        self: &Arc<Self>,
        name: String,
        handler_type: HandlerType,
    ) -> Arc<HandlerMetrics> {
        let metrics = HandlerMetrics::new(name, handler_type);
        self.handlers.write().await.push(metrics.clone());
        metrics
    }

    pub async fn get_all_statuses(&self) -> Vec<HandlerStatus> {
        let handlers = self.handlers.read().await;
        let mut statuses = Vec::with_capacity(handlers.len());

        for handler in handlers.iter() {
            statuses.push(handler.get_status().await);
        }

        statuses
    }

    pub async fn get_unhealthy_handlers(&self) -> Vec<HandlerStatus> {
        self.get_all_statuses()
            .await
            .into_iter()
            .filter(|status| !matches!(status.health_status, HealthStatus::Healthy))
            .collect()
    }
}
