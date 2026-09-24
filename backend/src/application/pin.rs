use crate::application::state::AppState;
use crate::domain::message::ChatMessage;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PinCommand {
    Pin { message: Box<ChatMessage> },
    Unpin,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PinEvent {
    pub r#type: String,
    pub revision: i64,
    pub message: Option<ChatMessage>,
}

pub type PinSnapshot = (i64, Option<ChatMessage>);
pub type PinFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, PinError>> + Send + 'a>>;

#[derive(Debug, PartialEq, Eq)]
pub enum PinError {
    InvalidCommand,
    MissingWidget,
    Storage,
    Serialization,
}

pub trait PinStorage: Sync {
    fn save<'a>(
        &'a self,
        widget_id: &'a Uuid,
        message: Option<&'a ChatMessage>,
    ) -> PinFuture<'a, Option<i64>>;
    fn load<'a>(&'a self, widget_id: &'a Uuid) -> PinFuture<'a, Option<PinSnapshot>>;
}

pub async fn transition<S: PinStorage>(
    state: &AppState,
    storage: &S,
    widget_id: &str,
    widget_uuid: &Uuid,
    command: PinCommand,
) -> Result<PinEvent, PinError> {
    if !validate_command(&command, widget_id) {
        return Err(PinError::InvalidCommand);
    }
    let lock = state.pin_lock(widget_id).await;
    let _guard = lock.lock().await;
    let message = match command {
        PinCommand::Pin { message } => Some(*message),
        PinCommand::Unpin => None,
    };
    let revision = storage
        .save(widget_uuid, message.as_ref())
        .await?
        .ok_or(PinError::MissingWidget)?;
    let event = PinEvent {
        r#type: if message.is_some() {
            "pin_message"
        } else {
            "unpin_message"
        }
        .to_string(),
        revision,
        message,
    };
    let json = serde_json::to_string(&event).map_err(|_| PinError::Serialization)?;
    state.publish_widget(widget_id, json);
    Ok(event)
}

pub async fn snapshot<S: PinStorage>(
    state: &AppState,
    storage: &S,
    widget_id: &str,
    widget_uuid: &Uuid,
) -> Result<(PinEvent, String), PinError> {
    let lock = state.pin_lock(widget_id).await;
    let _guard = lock.lock().await;
    let (revision, message) = storage
        .load(widget_uuid)
        .await?
        .ok_or(PinError::MissingWidget)?;
    let event = PinEvent {
        r#type: "pin_state".to_string(),
        revision,
        message,
    };
    let json = serde_json::to_string(&event).map_err(|_| PinError::Serialization)?;
    Ok((event, json))
}

pub fn validate_command(command: &PinCommand, widget_id: &str) -> bool {
    match command {
        PinCommand::Unpin => true,
        PinCommand::Pin { message } => {
            message.r#type == "chat_message"
                && message.widget_id.as_deref() == Some(widget_id)
                && !message.id.trim().is_empty()
                && message.id.len() <= 256
                && !message.author.trim().is_empty()
                && message.author.len() <= 128
                && !message.content.trim().is_empty()
                && message.content.len() <= 2000
                && message.fragments.len() <= 100
                && message
                    .platform
                    .as_ref()
                    .is_some_and(|p| matches!(p.as_str(), "twitch" | "youtube" | "kick"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::state::AppState;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };
    use tokio::sync::Notify;

    #[derive(Default)]
    struct MemoryPins {
        pins: Mutex<std::collections::HashMap<Uuid, PinSnapshot>>,
        fail_save: AtomicBool,
        fail_load: AtomicBool,
        block_load: AtomicBool,
        load_started: Notify,
        release_load: Notify,
    }

    impl PinStorage for MemoryPins {
        fn save<'a>(
            &'a self,
            widget_id: &'a uuid::Uuid,
            message: Option<&'a ChatMessage>,
        ) -> PinFuture<'a, Option<i64>> {
            Box::pin(async move {
                if self.fail_save.load(Ordering::SeqCst) {
                    return Err(PinError::Storage);
                }
                let mut pins = self.pins.lock().unwrap();
                let pin = pins.get_mut(widget_id);
                let Some(pin) = pin else { return Ok(None) };
                pin.0 += 1;
                pin.1 = message.cloned();
                Ok(Some(pin.0))
            })
        }

        fn load<'a>(&'a self, widget_id: &'a uuid::Uuid) -> PinFuture<'a, Option<PinSnapshot>> {
            Box::pin(async move {
                if self.block_load.load(Ordering::SeqCst) {
                    self.load_started.notify_one();
                    self.release_load.notified().await;
                }
                if self.fail_load.load(Ordering::SeqCst) {
                    return Err(PinError::Storage);
                }
                Ok(self.pins.lock().unwrap().get(widget_id).cloned())
            })
        }
    }

    #[tokio::test]
    async fn valid_pin_persists_revision_and_publishes_widget_event() {
        let widget_id = uuid::Uuid::new_v4();
        let id = widget_id.to_string();
        let state = Arc::new(AppState::default());
        let mut connection = state.acquire_widget(&id, false);
        let mut receiver = connection.take_receiver();
        let storage = MemoryPins::default();
        storage.pins.lock().unwrap().insert(widget_id, (0, None));
        let mut message = ChatMessage::mock(&id);
        message.platform = Some("twitch".to_string());

        let event = transition(
            &state,
            &storage,
            &id,
            &widget_id,
            PinCommand::Pin {
                message: Box::new(message.clone()),
            },
        )
        .await
        .unwrap();

        assert_eq!(event.r#type, "pin_message");
        assert_eq!(event.revision, 1);
        assert_eq!(event.message.as_ref().unwrap().id, message.id);
        assert_eq!(storage.pins.lock().unwrap().get(&widget_id).unwrap().0, 1);
        let received: PinEvent = serde_json::from_str(&receiver.recv().await.unwrap()).unwrap();
        assert_eq!(received.r#type, "pin_message");
        assert_eq!(received.revision, 1);
        assert_eq!(received.message.unwrap().id, message.id);
    }

    fn valid_message(id: &str, content: &str) -> ChatMessage {
        let mut message = ChatMessage::mock(id);
        message.platform = Some("twitch".to_string());
        message.content = content.to_string();
        message
    }

    #[tokio::test]
    async fn invalid_or_failed_pin_does_not_write_or_publish() {
        let widget = Uuid::new_v4();
        let id = widget.to_string();
        let state = Arc::new(AppState::default());
        let mut connection = state.acquire_widget(&id, false);
        let mut receiver = connection.take_receiver();
        let storage = MemoryPins::default();
        storage.pins.lock().unwrap().insert(widget, (0, None));
        let invalid = PinCommand::Pin {
            message: Box::new(valid_message("wrong", "hello")),
        };
        assert_eq!(
            transition(&state, &storage, &id, &widget, invalid)
                .await
                .unwrap_err(),
            PinError::InvalidCommand
        );
        assert_eq!(storage.pins.lock().unwrap().get(&widget).unwrap().0, 0);
        assert!(receiver.try_recv().is_err());

        storage.fail_save.store(true, Ordering::SeqCst);
        assert_eq!(
            transition(&state, &storage, &id, &widget, PinCommand::Unpin)
                .await
                .unwrap_err(),
            PinError::Storage
        );
        assert_eq!(storage.pins.lock().unwrap().get(&widget).unwrap().0, 0);
        assert!(receiver.try_recv().is_err());
    }

    #[tokio::test]
    async fn replacement_unpin_and_missing_widget_preserve_revision_and_scope() {
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        let first_id = first.to_string();
        let second_id = second.to_string();
        let state = Arc::new(AppState::default());
        let mut first_connection = state.acquire_widget(&first_id, false);
        let mut first_rx = first_connection.take_receiver();
        let mut second_connection = state.acquire_widget(&second_id, false);
        let mut second_rx = second_connection.take_receiver();
        let storage = MemoryPins::default();
        storage.pins.lock().unwrap().insert(first, (0, None));
        storage.pins.lock().unwrap().insert(second, (0, None));

        for (revision, content) in [(1, "first"), (2, "replacement")] {
            let event = transition(
                &state,
                &storage,
                &first_id,
                &first,
                PinCommand::Pin {
                    message: Box::new(valid_message(&first_id, content)),
                },
            )
            .await
            .unwrap();
            assert_eq!(event.revision, revision);
            assert_eq!(event.message.unwrap().content, content);
            let received: PinEvent = serde_json::from_str(&first_rx.recv().await.unwrap()).unwrap();
            assert_eq!(received.revision, revision);
        }
        let unpin = transition(&state, &storage, &first_id, &first, PinCommand::Unpin)
            .await
            .unwrap();
        assert_eq!(unpin.r#type, "unpin_message");
        assert_eq!(unpin.revision, 3);
        assert!(unpin.message.is_none());
        let received: PinEvent = serde_json::from_str(&first_rx.recv().await.unwrap()).unwrap();
        assert_eq!(received.r#type, "unpin_message");
        assert!(received.message.is_none());
        assert!(second_rx.try_recv().is_err());
        assert_eq!(storage.pins.lock().unwrap().get(&second).unwrap().0, 0);

        let other = transition(&state, &storage, &second_id, &second, PinCommand::Unpin)
            .await
            .unwrap();
        assert_eq!(other.revision, 1);
        assert!(first_rx.try_recv().is_err());
        assert_eq!(
            transition(
                &state,
                &storage,
                &first_id,
                &Uuid::new_v4(),
                PinCommand::Unpin
            )
            .await
            .unwrap_err(),
            PinError::MissingWidget
        );
        assert!(first_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn snapshot_reflects_persisted_revision_and_load_failures() {
        let widget = Uuid::new_v4();
        let id = widget.to_string();
        let state = AppState::default();
        let storage = MemoryPins::default();
        storage
            .pins
            .lock()
            .unwrap()
            .insert(widget, (7, Some(valid_message(&id, "saved"))));
        let (event, json) = snapshot(&state, &storage, &id, &widget).await.unwrap();
        assert_eq!(event.r#type, "pin_state");
        assert_eq!(event.revision, 7);
        assert_eq!(event.message.unwrap().content, "saved");
        assert_eq!(serde_json::from_str::<PinEvent>(&json).unwrap().revision, 7);
        storage.pins.lock().unwrap().insert(widget, (8, None));
        assert!(snapshot(&state, &storage, &id, &widget)
            .await
            .unwrap()
            .0
            .message
            .is_none());
        storage.fail_load.store(true, Ordering::SeqCst);
        assert_eq!(
            snapshot(&state, &storage, &id, &widget).await.unwrap_err(),
            PinError::Storage
        );
        storage.fail_load.store(false, Ordering::SeqCst);
        assert_eq!(
            snapshot(&state, &storage, &id, &Uuid::new_v4())
                .await
                .unwrap_err(),
            PinError::MissingWidget
        );
    }

    #[tokio::test]
    async fn subscribed_snapshot_precedes_transition_queued_behind_load() {
        let widget = Uuid::new_v4();
        let id = widget.to_string();
        let state = Arc::new(AppState::default());
        let mut connection = state.acquire_widget(&id, false);
        let mut receiver = connection.take_receiver();
        let storage = Arc::new(MemoryPins::default());
        storage.pins.lock().unwrap().insert(widget, (4, None));
        storage.block_load.store(true, Ordering::SeqCst);
        let snapshot_state = state.clone();
        let snapshot_storage = storage.clone();
        let snapshot_id = id.clone();
        let snapshot_task = tokio::spawn(async move {
            snapshot(&snapshot_state, &*snapshot_storage, &snapshot_id, &widget)
                .await
                .unwrap()
                .1
        });
        storage.load_started.notified().await;
        let transition_state = state.clone();
        let transition_storage = storage.clone();
        let transition_id = id.clone();
        let transition_task = tokio::spawn(async move {
            transition(
                &transition_state,
                &*transition_storage,
                &transition_id,
                &widget,
                PinCommand::Unpin,
            )
            .await
            .unwrap()
        });
        storage.release_load.notify_one();
        let first_frame: PinEvent = serde_json::from_str(&snapshot_task.await.unwrap()).unwrap();
        let live = transition_task.await.unwrap();
        let second_frame: PinEvent = serde_json::from_str(&receiver.recv().await.unwrap()).unwrap();
        assert_eq!(
            (first_frame.r#type.as_str(), first_frame.revision),
            ("pin_state", 4)
        );
        assert_eq!((live.revision, second_frame.revision), (5, 5));
    }

    #[test]
    fn pin_accepts_normalized_message_for_widget() {
        let id = uuid::Uuid::new_v4().to_string();
        let mut message = ChatMessage::mock(&id);
        message.platform = Some("twitch".to_string());
        assert!(validate_command(
            &PinCommand::Pin {
                message: Box::new(message.clone())
            },
            &id
        ));
        assert!(!validate_command(
            &PinCommand::Pin {
                message: Box::new(message)
            },
            &uuid::Uuid::new_v4().to_string()
        ));
    }

    #[test]
    fn pin_rejects_invalid_content_and_platform() {
        let id = uuid::Uuid::new_v4().to_string();
        let mut message = ChatMessage::mock(&id);
        message.content.clear();
        assert!(!validate_command(
            &PinCommand::Pin {
                message: Box::new(message)
            },
            &id
        ));
        let message = ChatMessage::mock(&id);
        assert!(!validate_command(
            &PinCommand::Pin {
                message: Box::new(message)
            },
            &id
        ));
    }
}
