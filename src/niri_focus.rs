use kameo::actor::{Actor, ActorRef, Spawn};
use kameo::error::Infallible;
use kameo::message::{Context, Message};

use crate::error::{Error, Result};
use crate::{FocusObservation, FocusTracker, NiriEvent, NiriWindowId, SystemTarget};

#[derive(Debug)]
pub struct NiriFocus {
    tracker: FocusTracker,
    applied_event_count: u64,
    emitted_observation_count: u64,
}

impl NiriFocus {
    pub fn new(target: SystemTarget, id: NiriWindowId) -> Self {
        Self::from_tracker(FocusTracker::new(target, id))
    }

    pub fn from_tracker(tracker: FocusTracker) -> Self {
        Self {
            tracker,
            applied_event_count: 0,
            emitted_observation_count: 0,
        }
    }

    pub async fn start(focus: Self) -> ActorRef<Self> {
        let reference = Self::spawn(focus);
        reference.wait_for_startup().await;
        reference
    }

    pub async fn stop(reference: ActorRef<Self>) -> Result<()> {
        reference
            .stop_gracefully()
            .await
            .map_err(|error| Error::ActorCall {
                detail: error.to_string(),
            })?;
        reference.wait_for_shutdown().await;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyNiriEvent {
    pub event: NiriEvent,
}

impl Actor for NiriFocus {
    type Args = Self;
    type Error = Infallible;

    async fn on_start(
        focus: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(focus)
    }
}

impl Message<ApplyNiriEvent> for NiriFocus {
    type Reply = Vec<FocusObservation>;

    async fn handle(
        &mut self,
        message: ApplyNiriEvent,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.applied_event_count = self.applied_event_count.saturating_add(1);
        let observations = self.tracker.apply_event(&message.event);
        self.emitted_observation_count = self
            .emitted_observation_count
            .saturating_add(observations.len() as u64);
        observations
    }
}
