use tokio::sync::{broadcast, watch};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TunnelStatus {
    Connected,
    Reconnecting,
    Disconnected,
}

pub struct TunnelHandle {
    pub(crate) shutdown_tx: broadcast::Sender<bool>,
    pub(crate) status_rx: watch::Receiver<TunnelStatus>,
}

impl TunnelHandle {
    pub fn stop(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    pub fn status(&self) -> TunnelStatus {
        *self.status_rx.borrow()
    }

    pub async fn wait_status_change(&mut self) -> Result<TunnelStatus, watch::error::RecvError> {
        self.status_rx.changed().await?;
        Ok(self.status())
    }
}
