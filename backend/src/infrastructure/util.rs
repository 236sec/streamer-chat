use std::time::Duration;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

pub async fn sleep_or_cancel(
    duration: Duration,
    tx: &broadcast::Sender<String>,
    cancel: &CancellationToken,
) -> bool {
    let mut elapsed = Duration::ZERO;
    let step = Duration::from_millis(100);
    let mut zero_receivers_count = 0;
    while elapsed < duration {
        if cancel.is_cancelled() {
            return false;
        }
        if tx.receiver_count() == 0 {
            zero_receivers_count += 1;
            if zero_receivers_count >= 10 {
                return false;
            }
        } else {
            zero_receivers_count = 0;
        }
        let current_step = step.min(duration - elapsed);
        tokio::time::sleep(current_step).await;
        elapsed += current_step;
    }
    !cancel.is_cancelled() && (tx.receiver_count() > 0 || zero_receivers_count < 10)
}
