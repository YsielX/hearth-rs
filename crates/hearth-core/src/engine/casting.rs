use super::*;

impl<R: CardRuntime> Game<R> {
    pub(super) fn stage_existing_spell_cast(
        &mut self,
        source: EntityId,
        card: EntityId,
        mut target: Option<EntityId>,
        skip_if_invalid: bool,
        random_target: bool,
        choice_policy: ChoicePolicy,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let entity = self
            .state
            .entity(card)
            .cloned()
            .ok_or(GameError::UnknownEntity(card))?;
        if entity.kind != CardKind::Spell {
            return Err(GameError::CardCannotBeCast(entity.card_id));
        }
        if matches!(
            entity.zone,
            Zone::Hero | Zone::Board | Zone::Weapon | Zone::HeroPower | Zone::Secret
        ) {
            return Ok(());
        }
        let player = entity.controller;
        let definition = self.runtime.definition(&entity.card_id).unwrap();
        let definition_secret = definition.secret;
        let target_mode = definition.target_mode;
        let is_secret =
            definition_secret || self.keyword_bool(card, "enters_secret_zone", false, None)?;
        if is_secret && self.state.player(player).secrets.len() >= MAX_SECRET_SIZE {
            return Ok(());
        }

        self.state.entities.get_mut(&card).unwrap().choice_policy = choice_policy;
        let valid_targets = self.valid_targets(card)?;
        let hero = self.state.player(player).hero;
        let random_target =
            random_target || self.keyword_bool(hero, "randomize_targets", false, Some(card))?;
        let mut random_effects = Vec::new();
        if random_target && !valid_targets.is_empty() {
            let index = self.rng.random_range(0..valid_targets.len());
            self.state.random_counter = self.state.random_counter.saturating_add(1);
            target = Some(valid_targets[index]);
            random_effects = self.publish(GameEvent::RandomChoiceMade {
                source,
                index,
                options: valid_targets.len(),
            })?;
        }

        self.remove_from_zone(card, entity.zone, player);
        self.state.entities.get_mut(&card).unwrap().zone = Zone::SetAside;
        if target_mode.requires_target(valid_targets.len()) && target.is_none() {
            if skip_if_invalid {
                self.state.entities.get_mut(&card).unwrap().zone = Zone::Removed;
                return Ok(());
            }
            return Err(GameError::TargetRequired);
        }
        if let Some(selected) = target
            && !valid_targets.contains(&selected)
        {
            if skip_if_invalid {
                self.state.entities.get_mut(&card).unwrap().zone = Zone::Removed;
                return Ok(());
            }
            return Err(GameError::InvalidTarget(selected));
        }

        if is_secret {
            self.move_to_secret(card, player);
        } else {
            self.move_to_graveyard(card, player);
        }
        self.refresh_auras()?;
        let mut items = random_effects
            .into_iter()
            .map(ResolutionItem::Effect)
            .collect::<Vec<_>>();
        if let Some(selected) = target {
            let targeted = self.begin_event(GameEvent::SpellTargeted {
                player,
                spell: card,
                target: selected,
                generated_by: Some(source),
            })?;
            items.extend(
                self.publish_after(targeted.id, targeted.event.clone())?
                    .into_iter()
                    .map(ResolutionItem::Effect),
            );
            items.push(ResolutionItem::ResolveEffectSpell {
                target_event: targeted,
                generated_by: source,
                secret: is_secret,
                declared_target: selected,
            });
            for item in items.into_iter().rev() {
                queue.push_front(item);
            }
            return Ok(());
        }
        items.extend(
            self.runtime
                .on_play(&self.state, card, target)
                .map_err(GameError::Script)?
                .into_iter()
                .map(ResolutionItem::Effect),
        );
        if is_secret {
            let secret = self.begin_event(GameEvent::SecretPlayed {
                player,
                secret: card,
            })?;
            items.push(ResolutionItem::PublishAfter {
                id: secret.id,
                event: secret.event,
            });
        }
        let cast = self.begin_event(GameEvent::SpellCast {
            player,
            spell: card,
            generated_by: Some(source),
            target,
            cost: 0,
            target_was_friendly_minion: false,
        })?;
        items.push(ResolutionItem::PublishAfter {
            id: cast.id,
            event: cast.event,
        });
        for item in items.into_iter().rev() {
            queue.push_front(item);
        }
        Ok(())
    }

    pub(super) fn resolve_effect_spell(
        &mut self,
        target_event: PendingEvent,
        generated_by: EntityId,
        secret: bool,
        declared_target: EntityId,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let GameEvent::SpellTargeted {
            player,
            spell,
            target,
            generated_by: Some(event_source),
        } = target_event.event
        else {
            return Err(GameError::EventSpellTargetNotReplaceable(target_event.id));
        };
        if event_source != generated_by {
            return Err(GameError::EventSpellTargetNotReplaceable(target_event.id));
        }
        let mut items = self
            .runtime
            .on_play(&self.state, spell, Some(target))
            .map_err(GameError::Script)?
            .into_iter()
            .map(ResolutionItem::Effect)
            .collect::<Vec<_>>();
        if secret {
            let event = self.begin_event(GameEvent::SecretPlayed {
                player,
                secret: spell,
            })?;
            items.push(ResolutionItem::PublishAfter {
                id: event.id,
                event: event.event,
            });
        }
        let cast = self.begin_event(GameEvent::SpellCast {
            player,
            spell,
            generated_by: Some(generated_by),
            target: Some(declared_target),
            cost: 0,
            target_was_friendly_minion: false,
        })?;
        items.push(ResolutionItem::PublishAfter {
            id: cast.id,
            event: cast.event,
        });
        for item in items.into_iter().rev() {
            queue.push_front(item);
        }
        Ok(())
    }
}
