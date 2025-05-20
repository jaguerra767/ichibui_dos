use anyhow::Result;
use libra::{scale::ConnectedScale, Grams, MedianGrams};
use tokio::sync::oneshot;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::mpsc::{Receiver, Sender};

use crate::Config;

pub enum ScaleRequest {
    GetGrams {
        reply_to: oneshot::Sender<f64>,
    },
    GetMedianGrams {
        samples: usize,
        interval: Duration,
        reply_to: oneshot::Sender<f64>,
    },
}

pub async fn get_grams(tx: Sender<ScaleRequest>) -> Result<f64> {
    let (reply_tx, reply_rx) = oneshot::channel::<f64>();
    let _ = tx.send(ScaleRequest::GetGrams { reply_to: reply_tx }).await;
    let grams = reply_rx.await?;
    Ok(grams)
}

pub async fn get_median_grams(
    tx: Sender<ScaleRequest>,
    samples: usize,
    interval: Duration,
) -> Result<f64> {
    let (reply_tx, reply_rx) = oneshot::channel::<f64>();
    let _ = tx
        .send(ScaleRequest::GetMedianGrams {
            samples,
            interval,
            reply_to: reply_tx,
        })
        .await;
    let grams = reply_rx.await?;
    Ok(grams)
}

pub async fn scale_task(mut rx: Receiver<ScaleRequest>, interval: Duration) -> Result<()> {
    let config = Config::load();
    let timeout = config.phidget.timeout;
    let scale = connect_scale(timeout).await?;
    let scale = Arc::new(Mutex::new(scale));
    set_data_intervals(scale.clone(), interval).await?;
    while let Some(cmd) = rx.recv().await {
        match cmd {
            ScaleRequest::GetGrams { reply_to } => {
                let grams = read(scale.clone()).await?;
                let _ = reply_to.send(grams.0);
            }
            ScaleRequest::GetMedianGrams {
                samples,
                interval,
                reply_to,
            } => {
                let grams = read_medians(scale.clone(), samples, interval).await?;
                let _ = reply_to.send(grams.0);
            }
        }
    }
    Ok(())
}

async fn read(scale: Arc<Mutex<ConnectedScale>>) -> Result<Grams> {
    let scale = scale.clone();
    let weight =
        tauri::async_runtime::spawn_blocking(move || scale.lock().unwrap().get_weight()).await??;
    Ok(weight)
}

async fn read_medians(
    scale: Arc<Mutex<ConnectedScale>>,
    samples: usize,
    interval: Duration,
) -> Result<MedianGrams> {
    let scale = scale.clone();
    let weight = tauri::async_runtime::spawn_blocking(move || {
        scale.lock().unwrap().get_median_weight(samples, interval)
    })
    .await??;
    Ok(weight)
}

async fn connect_scale(timeout: Duration) -> Result<ConnectedScale> {
    tauri::async_runtime::spawn_blocking(move || {
        let scale = ConnectedScale::without_id(timeout);
        scale
    })
    .await?
    .map_err(anyhow::Error::from)
}

async fn set_data_intervals(scale: Arc<Mutex<ConnectedScale>>, interval: Duration) -> Result<()> {
    let scale = scale.clone();
    tauri::async_runtime::spawn_blocking(move || {
        scale.lock().unwrap().set_data_intervals(interval)
    })
    .await??;
    Ok(())
}
