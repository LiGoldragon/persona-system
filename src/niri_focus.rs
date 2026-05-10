use kameo::actor::{Actor, ActorRef, Spawn};
use kameo::error::Infallible;
use kameo::message::{Context, Message};

use crate::error::{Error, Result};
use crate::{FocusObservation, FocusTracker, NiriEvent};

impl FocusTracker {
    pub async fn start(tracker: Self) -> ActorRef<Self> {
        let reference = Self::spawn(tracker);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusStatisticsProbe {
    minimum_applied_event_count: u64,
    minimum_emitted_observation_count: u64,
}

impl FocusStatisticsProbe {
    pub fn expecting_at_least(
        minimum_applied_event_count: u64,
        minimum_emitted_observation_count: u64,
    ) -> Self {
        Self {
            minimum_applied_event_count,
            minimum_emitted_observation_count,
        }
    }

    fn inspect(self, applied_event_count: u64, emitted_observation_count: u64) -> FocusStatistics {
        FocusStatistics {
            applied_event_count,
            emitted_observation_count,
            minimum_applied_event_count: self.minimum_applied_event_count,
            minimum_emitted_observation_count: self.minimum_emitted_observation_count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadFocusStatistics {
    pub probe: FocusStatisticsProbe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, kameo::Reply)]
pub struct FocusStatistics {
    applied_event_count: u64,
    emitted_observation_count: u64,
    minimum_applied_event_count: u64,
    minimum_emitted_observation_count: u64,
}

impl FocusStatistics {
    pub fn applied_event_count(&self) -> u64 {
        self.applied_event_count
    }

    pub fn emitted_observation_count(&self) -> u64 {
        self.emitted_observation_count
    }

    pub fn satisfied(&self) -> bool {
        self.applied_event_count >= self.minimum_applied_event_count
            && self.emitted_observation_count >= self.minimum_emitted_observation_count
    }
}

impl Actor for FocusTracker {
    type Args = Self;
    type Error = Infallible;

    async fn on_start(
        tracker: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(tracker)
    }
}

impl Message<ApplyNiriEvent> for FocusTracker {
    type Reply = Vec<FocusObservation>;

    async fn handle(
        &mut self,
        message: ApplyNiriEvent,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.apply_event_from_mailbox(&message.event)
    }
}

impl Message<ReadFocusStatistics> for FocusTracker {
    type Reply = FocusStatistics;

    async fn handle(
        &mut self,
        message: ReadFocusStatistics,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        message
            .probe
            .inspect(self.applied_event_count(), self.emitted_observation_count())
    }
}
