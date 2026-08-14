use super::*;

impl<R: CardRuntime> Game<R> {
    pub(super) fn apply_stat_batch(
        &mut self,
        source: EntityId,
        modifications: Vec<crate::EntityStatModification>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let mut seen = std::collections::BTreeSet::new();
        for modification in modifications {
            if !seen.insert(modification.target) {
                continue;
            }
            let expires_at = self.expiry_for(modification.duration);
            let Some(entity) = self.state.entities.get_mut(&modification.target) else {
                continue;
            };
            if entity.kind == CardKind::Location
                && modification
                    .modifiers
                    .iter()
                    .any(|modifier| modifier.stat != Stat::Cost)
            {
                continue;
            }
            let id = EnchantmentId(self.state.next_enchantment_id);
            self.state.next_enchantment_id += 1;
            entity.enchantments.push(Enchantment {
                id,
                source,
                attack: 0,
                health: 0,
                modifiers: modification.modifiers,
                keywords: Vec::new(),
                silenciable: modification.silenciable,
                expires_at,
            });
            Self::recompute_entity(entity);
            if modification.reset_damage {
                entity.damage = 0;
            }
        }
        self.refresh_auras()?;
        Ok(Vec::new())
    }
}
