use super::*;

impl<R: CardRuntime> Game<R> {
    /// Returns true when resolution paused for player input.
    pub(super) fn resolve_effect_item(
        &mut self,
        effect: EffectSpec,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<bool, GameError> {
        match effect {
            EffectSpec::RequestChoice {
                player,
                source,
                prompt,
                options,
                resume_hook,
            } => {
                if options.is_empty() {
                    return Err(GameError::EmptyChoice);
                }
                if options.len() > MAX_CHOICE_OPTIONS {
                    return Err(GameError::TooManyChoiceOptions {
                        options: options.len(),
                    });
                }
                if prompt.is_empty() || prompt.len() > MAX_CHOICE_PROMPT_BYTES {
                    return Err(GameError::InvalidChoicePrompt);
                }
                if resume_hook.is_empty() || resume_hook.len() > 64 {
                    return Err(GameError::InvalidContinuationHook);
                }
                for option in &options {
                    if option.label.is_empty() || option.label.len() > MAX_CHOICE_LABEL_BYTES {
                        return Err(GameError::InvalidChoiceLabel);
                    }
                    option
                        .value
                        .validate()
                        .map_err(GameError::InvalidChoiceValue)?;
                }
                if !self.collect_deaths().is_empty() || self.any_hero_dead() {
                    queue.push_front(ResolutionItem::Effect(EffectSpec::RequestChoice {
                        player,
                        source,
                        prompt,
                        options,
                        resume_hook,
                    }));
                    queue.push_front(ResolutionItem::DeathCheck);
                    return Ok(false);
                }
                let option_count = options.len();
                self.state.pending_input = Some(crate::PendingInput {
                    player,
                    source,
                    prompt,
                    options,
                    resume_hook,
                    remaining_resolution: queue.drain(..).collect(),
                });
                self.state.log.push(GameEvent::ChoiceRequested {
                    player,
                    source,
                    options: option_count,
                });
                Ok(true)
            }
            EffectSpec::RandomChoice {
                source,
                options,
                resume_hook,
            } => {
                if options.is_empty() {
                    return Err(GameError::EmptyRandomChoice);
                }
                if options.len() > MAX_CHOICE_OPTIONS {
                    return Err(GameError::TooManyChoiceOptions {
                        options: options.len(),
                    });
                }
                if resume_hook.is_empty() || resume_hook.len() > 64 {
                    return Err(GameError::InvalidContinuationHook);
                }
                for option in &options {
                    option.validate().map_err(GameError::InvalidChoiceValue)?;
                }
                let index = self.rng.random_range(0..options.len());
                self.state.random_counter += 1;
                let choice = options[index].clone();
                let mut generated = self.publish(GameEvent::RandomChoiceMade {
                    source,
                    index,
                    options: options.len(),
                })?;
                generated.extend(
                    self.runtime
                        .on_resume(&self.state, source, &resume_hook, &choice)
                        .map_err(GameError::Script)?,
                );
                Self::prepend_effects(queue, generated);
                Ok(false)
            }
            EffectSpec::DiscoverCards {
                player,
                source,
                prompt,
                candidates,
                count,
                resume_hook,
            } => {
                if count == 0 {
                    return Err(GameError::InvalidDiscoverCount);
                }
                if !self.collect_deaths().is_empty() || self.any_hero_dead() {
                    queue.push_front(ResolutionItem::Effect(EffectSpec::DiscoverCards {
                        player,
                        source,
                        prompt,
                        candidates,
                        count,
                        resume_hook,
                    }));
                    queue.push_front(ResolutionItem::DeathCheck);
                    return Ok(false);
                }
                let mut seen = std::collections::BTreeSet::new();
                let mut pool = Vec::new();
                for card_id in candidates {
                    if !seen.insert(card_id.clone()) {
                        continue;
                    }
                    if self.runtime.definition(&card_id).is_none() {
                        return Err(GameError::UnknownCard(card_id));
                    }
                    pool.push(card_id);
                }
                if pool.is_empty() {
                    return Err(GameError::EmptyDiscoverPool);
                }
                let population = pool.len();
                let mut sampled = Vec::with_capacity(count.min(population));
                for _ in 0..count.min(population) {
                    let index = self.rng.random_range(0..pool.len());
                    self.state.random_counter += 1;
                    sampled.push(pool.remove(index));
                }
                let options = sampled
                    .iter()
                    .map(|card_id| {
                        let definition = self.runtime.definition(card_id).unwrap();
                        ChoiceOption {
                            label: format!("{} [{}]", definition.name, card_id),
                            value: crate::ChoiceValue::Card(card_id.clone()),
                        }
                    })
                    .collect();
                let triggered = self.publish(GameEvent::RandomCardsSampled {
                    source,
                    cards: sampled,
                    population,
                })?;
                queue.push_front(ResolutionItem::Effect(EffectSpec::RequestChoice {
                    player,
                    source,
                    prompt,
                    options,
                    resume_hook,
                }));
                Self::prepend_effects(queue, triggered);
                Ok(false)
            }
            EffectSpec::DiscoverEntities {
                player,
                source,
                prompt,
                candidates,
                count,
                resume_hook,
            } => {
                if count == 0 {
                    return Err(GameError::InvalidDiscoverCount);
                }
                if !self.collect_deaths().is_empty() || self.any_hero_dead() {
                    queue.push_front(ResolutionItem::Effect(EffectSpec::DiscoverEntities {
                        player,
                        source,
                        prompt,
                        candidates,
                        count,
                        resume_hook,
                    }));
                    queue.push_front(ResolutionItem::DeathCheck);
                    return Ok(false);
                }
                let mut seen = std::collections::BTreeSet::new();
                let mut pool = Vec::new();
                for entity in candidates {
                    if !seen.insert(entity) {
                        continue;
                    }
                    if self.state.entity(entity).is_none() {
                        return Err(GameError::UnknownEntity(entity));
                    }
                    pool.push(entity);
                }
                if pool.is_empty() {
                    return Err(GameError::EmptyDiscoverPool);
                }
                let population = pool.len();
                let mut sampled = Vec::with_capacity(count.min(population));
                for _ in 0..count {
                    if pool.is_empty() {
                        break;
                    }
                    let index = self.rng.random_range(0..pool.len());
                    self.state.random_counter += 1;
                    let selected = pool.remove(index);
                    let card_id = self.state.entities[&selected].card_id.clone();
                    sampled.push(selected);
                    // Discover never presents the same definition twice. Keeping actual
                    // entities in the sampling pool still weights a card by its copy count.
                    pool.retain(|entity| self.state.entities[entity].card_id != card_id);
                }
                let options = sampled
                    .iter()
                    .map(|entity| {
                        let entity_state = self.state.entity(*entity).unwrap();
                        ChoiceOption {
                            label: format!("{} [{}]", entity_state.name, entity),
                            value: crate::ChoiceValue::Entity(*entity),
                        }
                    })
                    .collect();
                let triggered = self.publish(GameEvent::RandomEntitiesSampled {
                    source,
                    entities: sampled,
                    population,
                })?;
                queue.push_front(ResolutionItem::Effect(EffectSpec::RequestChoice {
                    player,
                    source,
                    prompt,
                    options,
                    resume_hook,
                }));
                Self::prepend_effects(queue, triggered);
                Ok(false)
            }
            EffectSpec::Damage {
                source,
                target,
                amount,
            } => {
                let target_kind = self
                    .state
                    .entity(target)
                    .map(|entity| entity.kind)
                    .ok_or(GameError::UnknownEntity(target))?;
                if !matches!(target_kind, CardKind::Minion | CardKind::Hero) {
                    return Ok(false);
                }
                let amount = self.apply_spell_damage_bonus(source, amount);
                let pending = self.begin_event(GameEvent::Damaged {
                    source,
                    target,
                    amount,
                })?;
                let before = self.trigger_event(&pending, EventTiming::Before)?;
                queue.push_front(ResolutionItem::CommitEvent(pending));
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::CastSpell {
                source,
                player,
                card_id,
                target,
            } => {
                let definition = self
                    .runtime
                    .definition(&card_id)
                    .cloned()
                    .ok_or_else(|| GameError::UnknownCard(card_id.clone()))?;
                if definition.kind != CardKind::Spell {
                    return Err(GameError::CardCannotBeCast(card_id));
                }
                let spell = self.instantiate(&definition.id, player, Zone::SetAside)?;
                let is_secret = definition.secret
                    || self.keyword_bool(spell, "enters_secret_zone", false, None)?;
                if is_secret && self.state.player(player).secrets.len() >= MAX_SECRET_SIZE {
                    self.state.entities.get_mut(&spell).unwrap().zone = Zone::Removed;
                    return Ok(false);
                }

                let valid_targets = self.valid_targets(spell)?;
                if definition.target_mode.requires_target(valid_targets.len()) && target.is_none() {
                    return Err(GameError::TargetRequired);
                }
                if let Some(target) = target
                    && !valid_targets.contains(&target)
                {
                    return Err(GameError::InvalidTarget(target));
                }

                if is_secret {
                    self.move_to_secret(spell, player);
                } else {
                    self.move_to_graveyard(spell, player);
                }
                self.refresh_auras()?;

                let mut items = self
                    .runtime
                    .on_play(&self.state, spell, target)
                    .map_err(GameError::Script)?
                    .into_iter()
                    .map(ResolutionItem::Effect)
                    .collect::<Vec<_>>();
                if is_secret {
                    let secret = self.begin_event(GameEvent::SecretPlayed {
                        player,
                        secret: spell,
                    })?;
                    items.push(ResolutionItem::PublishAfter {
                        id: secret.id,
                        event: secret.event,
                    });
                }
                let cast = self.begin_event(GameEvent::SpellCast {
                    player,
                    spell,
                    generated_by: Some(source),
                })?;
                items.push(ResolutionItem::PublishAfter {
                    id: cast.id,
                    event: cast.event,
                });
                for item in items.into_iter().rev() {
                    queue.push_front(item);
                }
                Ok(false)
            }
            EffectSpec::CastDrawn { card } => {
                let entity = self
                    .state
                    .entity(card)
                    .cloned()
                    .ok_or(GameError::UnknownEntity(card))?;
                let player = entity.controller;
                if entity.zone != Zone::Hand || entity.kind != CardKind::Spell {
                    return Ok(false);
                }
                self.remove_from_zone(card, Zone::Hand, player);
                self.move_to_graveyard(card, player);
                self.refresh_auras()?;
                let mut items = self
                    .runtime
                    .on_play(&self.state, card, None)
                    .map_err(GameError::Script)?
                    .into_iter()
                    .map(ResolutionItem::Effect)
                    .collect::<Vec<_>>();
                let cast = self.begin_event(GameEvent::SpellCast {
                    player,
                    spell: card,
                    generated_by: Some(card),
                })?;
                items.push(ResolutionItem::PublishAfter {
                    id: cast.id,
                    event: cast.event,
                });
                for item in items.into_iter().rev() {
                    queue.push_front(item);
                }
                Ok(false)
            }
            EffectSpec::DamageGroup {
                source,
                targets,
                amount,
            } => {
                if targets.is_empty() {
                    return Ok(false);
                }
                let mut seen = std::collections::BTreeSet::new();
                let mut damage = Vec::new();
                let amount = self.apply_spell_damage_bonus(source, amount);
                for target in targets {
                    if !seen.insert(target) {
                        continue;
                    }
                    let target_kind = self
                        .state
                        .entity(target)
                        .map(|entity| entity.kind)
                        .ok_or(GameError::UnknownEntity(target))?;
                    if !matches!(target_kind, CardKind::Minion | CardKind::Hero) {
                        continue;
                    }
                    damage.push(self.begin_event(GameEvent::Damaged {
                        source,
                        target,
                        amount,
                    })?);
                }
                if damage.is_empty() {
                    return Ok(false);
                }
                let mut before = Vec::new();
                for pending in &damage {
                    before.extend(self.trigger_event(pending, EventTiming::Before)?);
                }
                queue.push_front(ResolutionItem::CommitDamageGroup { damage });
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::Heal {
                source,
                target,
                amount,
            } => {
                let target_kind = self
                    .state
                    .entity(target)
                    .map(|entity| entity.kind)
                    .ok_or(GameError::UnknownEntity(target))?;
                if !matches!(target_kind, CardKind::Minion | CardKind::Hero) {
                    return Ok(false);
                }
                let pending = self.begin_event(GameEvent::Healed {
                    source,
                    target,
                    amount: amount.max(0),
                })?;
                let before = self.trigger_event(&pending, EventTiming::Before)?;
                queue.push_front(ResolutionItem::CommitEvent(pending));
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::Draw { player, count } => {
                for _ in 0..count {
                    queue.push_front(ResolutionItem::DrawOne { player });
                }
                Ok(false)
            }
            EffectSpec::Discard {
                source,
                player,
                target,
            } => {
                if self.state.entity(target).is_none() {
                    return Err(GameError::UnknownEntity(target));
                }
                if !self.state.player(player).hand.contains(&target) {
                    return Ok(false);
                }
                let pending = self.begin_event(GameEvent::CardDiscarded {
                    source,
                    player,
                    card: target,
                })?;
                let before = self.trigger_event(&pending, EventTiming::Before)?;
                queue.push_front(ResolutionItem::CommitDiscard(pending));
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::Summon {
                player,
                card_id,
                position,
            } => {
                if self.state.player(player).board.len() >= MAX_BOARD_SIZE {
                    return Ok(false);
                }
                let max = self.state.player(player).board.len();
                if position.is_some_and(|position| position > max) {
                    return Err(GameError::InvalidBoardPosition {
                        position: position.unwrap(),
                        max,
                    });
                }
                let definition = self
                    .runtime
                    .definition(&card_id)
                    .ok_or_else(|| GameError::UnknownCard(card_id.clone()))?;
                if definition.kind != CardKind::Minion {
                    return Err(GameError::CardCannotBeSummoned(card_id));
                }
                let entity = self.instantiate(&card_id, player, Zone::SetAside)?;
                let pending = self.begin_event(GameEvent::MinionSummoned { player, entity })?;
                let before = self.trigger_event(&pending, EventTiming::Before)?;
                queue.push_front(ResolutionItem::CommitSummon {
                    summon: pending,
                    position,
                    origin: ReservedSummonOrigin::Generated,
                });
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::SummonFromHand { card } => {
                let entity = self
                    .state
                    .entity(card)
                    .cloned()
                    .ok_or(GameError::UnknownEntity(card))?;
                let player = entity.controller;
                if entity.zone != Zone::Hand
                    || entity.kind != CardKind::Minion
                    || self.state.player(player).board.len() >= MAX_BOARD_SIZE
                {
                    return Ok(false);
                }
                self.remove_from_zone(card, Zone::Hand, player);
                self.state.entities.get_mut(&card).unwrap().zone = Zone::SetAside;
                self.refresh_auras()?;
                let pending = self.begin_event(GameEvent::MinionSummoned {
                    player,
                    entity: card,
                })?;
                let before = self.trigger_event(&pending, EventTiming::Before)?;
                queue.push_front(ResolutionItem::CommitSummon {
                    summon: pending,
                    position: None,
                    origin: ReservedSummonOrigin::Generated,
                });
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::SummonCopy {
                player,
                target,
                position,
            } => {
                if self.state.player(player).board.len() >= MAX_BOARD_SIZE {
                    return Ok(false);
                }
                let max = self.state.player(player).board.len();
                if position.is_some_and(|position| position > max) {
                    return Err(GameError::InvalidBoardPosition {
                        position: position.unwrap(),
                        max,
                    });
                }
                let template = self
                    .state
                    .entity(target)
                    .cloned()
                    .ok_or(GameError::UnknownEntity(target))?;
                if template.zone != Zone::Board || template.kind != CardKind::Minion {
                    return Ok(false);
                }
                let entity = self.instantiate(&template.card_id, player, Zone::SetAside)?;
                self.copy_minion_state(&template, entity);
                let pending = self.begin_event(GameEvent::MinionSummoned { player, entity })?;
                let before = self.trigger_event(&pending, EventTiming::Before)?;
                queue.push_front(ResolutionItem::CommitSummon {
                    summon: pending,
                    position,
                    origin: ReservedSummonOrigin::Generated,
                });
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::SummonFreshCopy {
                source: _,
                player,
                target,
                position,
                health,
                without_keywords,
            } => {
                let entity = self
                    .state
                    .entity(target)
                    .ok_or(GameError::UnknownEntity(target))?;
                if entity.kind != CardKind::Minion {
                    return Ok(false);
                }
                queue.push_front(ResolutionItem::SummonFreshCopy {
                    player,
                    card_id: entity.card_id.clone(),
                    position: position.unwrap_or(self.state.player(player).board.len()),
                    health,
                    without_keywords,
                });
                Ok(false)
            }
            EffectSpec::Recruit {
                source: _,
                player,
                target,
                position,
            } => {
                if self.state.player(player).board.len() >= MAX_BOARD_SIZE {
                    return Ok(false);
                }
                let max = self.state.player(player).board.len();
                if position.is_some_and(|position| position > max) {
                    return Err(GameError::InvalidBoardPosition {
                        position: position.unwrap(),
                        max,
                    });
                }
                let entity = self
                    .state
                    .entity(target)
                    .ok_or(GameError::UnknownEntity(target))?;
                let Some(deck_position) = self
                    .state
                    .player(player)
                    .deck
                    .iter()
                    .position(|entity| *entity == target)
                else {
                    return Ok(false);
                };
                if entity.kind != CardKind::Minion
                    || entity.zone != Zone::Deck
                    || entity.controller != player
                {
                    return Err(GameError::EntityCannotBeRecruited(target));
                }
                let deck = &self.state.player(player).deck;
                let origin = ReservedSummonOrigin::Deck {
                    player,
                    position: deck_position,
                    previous: deck_position
                        .checked_sub(1)
                        .and_then(|index| deck.get(index).copied()),
                    next: deck.get(deck_position + 1).copied(),
                };
                let removed = self.state.player_mut(player).deck.remove(deck_position);
                debug_assert_eq!(removed, Some(target));
                self.state.entities.get_mut(&target).unwrap().zone = Zone::SetAside;
                self.refresh_auras()?;

                let pending = self.begin_event(GameEvent::MinionSummoned {
                    player,
                    entity: target,
                })?;
                let before = self.trigger_event(&pending, EventTiming::Before)?;
                queue.push_front(ResolutionItem::CommitSummon {
                    summon: pending,
                    position,
                    origin,
                });
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::MoveEntity {
                source: _,
                target,
                destination,
            } => {
                let entity = self
                    .state
                    .entity(target)
                    .ok_or(GameError::UnknownEntity(target))?;
                if matches!(
                    entity.zone,
                    Zone::Hero | Zone::HeroPower | Zone::SetAside | Zone::Removed
                ) {
                    return Err(GameError::EntityCannotMove {
                        entity: target,
                        zone: entity.zone,
                    });
                }
                let to = destination.zone();
                if entity.zone == to
                    && !matches!(
                        destination,
                        ZonePlacement::DeckTop
                            | ZonePlacement::DeckBottom
                            | ZonePlacement::DeckRandom
                    )
                {
                    return Ok(false);
                }
                let pending = self.begin_event(GameEvent::ZoneChanged {
                    entity: target,
                    from: entity.zone,
                    to,
                })?;
                let before = self.trigger_event(&pending, EventTiming::Before)?;
                queue.push_front(ResolutionItem::CommitZoneChange {
                    change: pending,
                    destination,
                });
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::ChangeController {
                source,
                target,
                player,
            } => {
                let entity = self
                    .state
                    .entity(target)
                    .ok_or(GameError::UnknownEntity(target))?;
                if entity.zone != Zone::Board
                    || entity.kind != CardKind::Minion
                    || entity.controller == player
                    || self.state.player(player).board.len() >= MAX_BOARD_SIZE
                {
                    return Ok(false);
                }
                let pending = self.begin_event(GameEvent::ControllerChanged {
                    source,
                    entity: target,
                    from: entity.controller,
                    to: player,
                })?;
                let before = self.trigger_event(&pending, EventTiming::Before)?;
                queue.push_front(ResolutionItem::CommitControllerChange(pending));
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::Transform {
                source,
                target,
                card_id,
            } => {
                let entity = self
                    .state
                    .entity(target)
                    .ok_or(GameError::UnknownEntity(target))?;
                if matches!(
                    entity.zone,
                    Zone::Hero | Zone::HeroPower | Zone::SetAside | Zone::Removed
                ) {
                    return Ok(false);
                }
                let definition = self
                    .runtime
                    .definition(&card_id)
                    .ok_or_else(|| GameError::UnknownCard(card_id.clone()))?;
                if definition.kind != entity.kind {
                    return Err(GameError::CardCannotTransformInto(card_id));
                }
                if entity.card_id == card_id {
                    return Ok(false);
                }
                let pending = self.begin_event(GameEvent::Transformed {
                    source,
                    entity: target,
                    from_card: entity.card_id.clone(),
                    to_card: card_id,
                })?;
                let before = self.trigger_event(&pending, EventTiming::Before)?;
                queue.push_front(ResolutionItem::CommitTransform(pending));
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::Continue {
                source,
                hook,
                payload,
            } => {
                if hook.is_empty() || hook.len() > 64 {
                    return Err(GameError::InvalidContinuationHook);
                }
                if let Some(payload) = &payload {
                    payload.validate().map_err(GameError::InvalidChoiceValue)?;
                }
                let generated = self
                    .runtime
                    .on_continue(&self.state, source, &hook, payload.as_ref())
                    .map_err(GameError::Script)?;
                Self::prepend_effects(queue, generated);
                Ok(false)
            }
            EffectSpec::CancelEvent { source: _, event } => {
                let pending = Self::find_pending_event_mut(queue, event)
                    .ok_or(GameError::EventNotPending(event))?;
                pending.cancelled = true;
                Ok(false)
            }
            EffectSpec::SetEventAmount {
                source: _,
                event,
                amount,
            } => {
                let pending = Self::find_pending_event_mut(queue, event)
                    .ok_or(GameError::EventNotPending(event))?;
                match &mut pending.event {
                    GameEvent::Damaged { amount: value, .. }
                    | GameEvent::Healed { amount: value, .. } => *value = amount.max(0),
                    GameEvent::Fatigue { amount: value, .. } => {
                        *value = u32::try_from(amount.max(0)).unwrap_or(u32::MAX)
                    }
                    _ => return Err(GameError::EventAmountNotReplaceable(event)),
                }
                Ok(false)
            }
            EffectSpec::SetTradeDraw {
                source: _,
                event,
                replacement,
            } => {
                let pending = Self::find_pending_event_mut(queue, event)
                    .ok_or(GameError::EventNotPending(event))?;
                match &mut pending.event {
                    GameEvent::TradeDraw {
                        replacement: selected,
                        ..
                    } => *selected = Some(replacement),
                    _ => return Err(GameError::EventTradeDrawNotReplaceable(event)),
                }
                Ok(false)
            }
            effect => {
                let events = self.apply_effect(effect)?;
                self.refresh_auras()?;
                queue.push_front(ResolutionItem::DeathCheck);
                for event in events.into_iter().rev() {
                    let triggered = self.publish(event)?;
                    Self::prepend_effects(queue, triggered);
                }
                Ok(false)
            }
        }
    }
}
