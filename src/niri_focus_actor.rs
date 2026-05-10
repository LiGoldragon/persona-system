use kameo::actor::{Actor, ActorRef, Spawn};
use kameo::error::Infallible;
use kameo::message::{Context, Message};

use crate::error::{Error, Result};
use crate::{FocusObservation, FocusTracker, NiriEvent, NiriWindowId, SystemTarget};

#[derive(Debug)]
pub struct NiriFocusActor {
    tracker: FocusTracker,
    applied_event_count: u64,
    emitted_observation_count: u64,
}

impl NiriFocusActor {
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyNiriEvent {
    pub event: NiriEvent,
}

#[derive(Debug, Clone)]
pub struct NiriFocusActorHandle {
    actor_reference: ActorRef<NiriFocusActor>,
}

impl NiriFocusActorHandle {
    pub async fn start(actor: NiriFocusActor) -> Self {
        let actor_reference = NiriFocusActor::spawn(actor);
        actor_reference.wait_for_startup().await;
        Self { actor_reference }
    }

    pub async fn apply(&self, event: NiriEvent) -> Result<Vec<FocusObservation>> {
        self.actor_reference
            .ask(ApplyNiriEvent { event })
            .await
            .map_err(|error| Error::ActorCall {
                detail: error.to_string(),
            })
    }

    pub async fn stop(self) -> Result<()> {
        self.actor_reference
            .stop_gracefully()
            .await
            .map_err(|error| Error::ActorCall {
                detail: error.to_string(),
            })?;
        self.actor_reference.wait_for_shutdown().await;
        Ok(())
    }
}

impl Actor for NiriFocusActor {
    type Args = Self;
    type Error = Infallible;

    async fn on_start(
        actor: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(actor)
    }
}

impl Message<ApplyNiriEvent> for NiriFocusActor {
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
