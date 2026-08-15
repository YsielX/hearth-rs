use std::collections::BTreeMap;

use hearth_core::{EntityId, PublicEntity};

use crate::{EntityRef, EnvError};

/// Episode-local entity identities for one player's observation stream.
/// Authoritative IDs never cross the adapter boundary.
#[derive(Default)]
pub(crate) struct EpisodeRefs {
    by_authoritative: BTreeMap<EntityId, EntityRef>,
}

impl EpisodeRefs {
    pub(crate) fn observe(&mut self, entity: EntityId) -> Result<EntityRef, EnvError> {
        if let Some(reference) = self.by_authoritative.get(&entity) {
            return Ok(*reference);
        }
        let reference = EntityRef(
            u16::try_from(self.by_authoritative.len())
                .map_err(|_| EnvError::TooManyObservedEntities)?,
        );
        self.by_authoritative.insert(entity, reference);
        Ok(reference)
    }

    pub(crate) fn observe_public(&mut self, entity: &PublicEntity) -> Result<EntityRef, EnvError> {
        self.observe(entity.id)
    }

    pub(crate) fn get(&self, entity: EntityId) -> Result<EntityRef, EnvError> {
        self.by_authoritative
            .get(&entity)
            .copied()
            .ok_or(EnvError::HiddenActionEntity(entity))
    }
}
