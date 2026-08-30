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
                continuation_owner,
            } => {
                if options.is_empty() {
                    let card_id = self
                        .state
                        .entity(source)
                        .map(|entity| entity.card_id.clone())
                        .unwrap_or_else(|| "<unknown>".to_owned());
                    return Err(GameError::EmptyChoice {
                        entity: source,
                        card_id,
                    });
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
                        continuation_owner,
                    }));
                    queue.push_front(ResolutionItem::DeathCheck);
                    return Ok(false);
                }
                let auto_random = self
                    .state
                    .entity(source)
                    .is_some_and(|entity| entity.choice_policy == ChoicePolicy::Random);
                if auto_random {
                    let index = self.rng.random_range(0..options.len());
                    self.state.random_counter = self.state.random_counter.saturating_add(1);
                    let choice = options[index].value.clone();
                    let mut generated = self.publish(GameEvent::RandomChoiceMade {
                        source,
                        index,
                        options: options.len(),
                    })?;
                    generated.extend(
                        self.runtime
                            .on_resume(
                                &self.state,
                                source,
                                continuation_owner.as_deref(),
                                &resume_hook,
                                &choice,
                            )
                            .map_err(GameError::Script)?,
                    );
                    Self::prepend_effects(queue, generated);
                    return Ok(false);
                }
                let option_count = options.len();
                self.state.pending_input = Some(crate::PendingInput {
                    player,
                    source,
                    prompt,
                    options,
                    resume_hook,
                    continuation_owner,
                    remaining_resolution: queue.drain(..).collect(),
                });
                self.state.record_event(GameEvent::ChoiceRequested {
                    player,
                    source,
                    options: option_count,
                });
                Ok(true)
            }
            EffectSpec::RandomChoice {
                source,
                mut options,
                resume_hook,
                continuation_owner,
            } => {
                if options.is_empty() {
                    return Err(GameError::EmptyRandomChoice);
                }
                if options.len() > MAX_RANDOM_CHOICE_OPTIONS {
                    return Err(GameError::TooManyRandomChoiceOptions {
                        options: options.len(),
                    });
                }
                if resume_hook.is_empty() || resume_hook.len() > 64 {
                    return Err(GameError::InvalidContinuationHook);
                }
                for option in &options {
                    option.validate().map_err(GameError::InvalidChoiceValue)?;
                }
                let mut selectable = Vec::with_capacity(options.len());
                for option in options.drain(..) {
                    if let ChoiceValue::Entity(entity) = &option
                        && !self.keyword_bool(
                            *entity,
                            "can_be_randomly_selected",
                            true,
                            Some(source),
                        )?
                    {
                        continue;
                    }
                    selectable.push(option);
                }
                options = selectable;
                if options.is_empty() {
                    return Ok(false);
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
                        .on_resume(
                            &self.state,
                            source,
                            continuation_owner.as_deref(),
                            &resume_hook,
                            &choice,
                        )
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
                continuation_owner,
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
                        continuation_owner,
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
                    continuation_owner,
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
                continuation_owner,
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
                        continuation_owner,
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
                            label: entity_state.name.clone(),
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
                    continuation_owner,
                }));
                Self::prepend_effects(queue, triggered);
                Ok(false)
            }
            EffectSpec::Damage {
                source,
                hits,
                apply_spell_damage,
            } => {
                if hits.is_empty() {
                    return Ok(false);
                }
                let mut seen = std::collections::BTreeSet::new();
                let mut damage = Vec::new();
                for (target, amount) in hits {
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
                    let amount = if apply_spell_damage {
                        self.apply_spell_damage_bonus(source, amount)
                    } else {
                        amount
                    };
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
            EffectSpec::CastSpell {
                source,
                player,
                card_id,
                target,
                skip_if_invalid,
                random_target,
                choice_policy,
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
                self.stage_existing_spell_cast(
                    source,
                    spell,
                    target,
                    skip_if_invalid,
                    random_target,
                    choice_policy,
                    queue,
                )?;
                Ok(false)
            }
            EffectSpec::CastExistingSpell {
                source,
                card,
                target,
                skip_if_invalid,
                random_target,
                choice_policy,
            } => {
                self.stage_existing_spell_cast(
                    source,
                    card,
                    target,
                    skip_if_invalid,
                    random_target,
                    choice_policy,
                    queue,
                )?;
                Ok(false)
            }
            EffectSpec::Heal { source, hits } => {
                if hits.is_empty() {
                    return Ok(false);
                }
                let source_player = self
                    .state
                    .entity(source)
                    .map(|entity| entity.controller)
                    .ok_or(GameError::UnknownEntity(source))?;
                let converts_healing = self.keyword_bool(
                    self.state.player(source_player).hero,
                    "healing_becomes_damage",
                    false,
                    Some(source),
                )?;
                let mut seen = std::collections::BTreeSet::new();
                let mut healing = Vec::new();
                for (target, amount) in hits {
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
                    let event = if converts_healing {
                        GameEvent::Damaged {
                            source,
                            target,
                            amount: amount.max(0),
                        }
                    } else {
                        GameEvent::Healed {
                            source,
                            target,
                            amount: amount.max(0),
                        }
                    };
                    healing.push(self.begin_event(event)?);
                }
                if healing.is_empty() {
                    return Ok(false);
                }
                let mut before = Vec::new();
                for pending in &healing {
                    before.extend(self.trigger_event(pending, EventTiming::Before)?);
                }
                if converts_healing {
                    queue.push_front(ResolutionItem::CommitDamageGroup { damage: healing });
                } else {
                    queue.push_front(ResolutionItem::CommitHealGroup { healing });
                }
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::Draw {
                source,
                player,
                count,
            } => {
                for _ in 0..count {
                    queue.push_front(ResolutionItem::DrawOne { player, source });
                }
                Ok(false)
            }
            EffectSpec::DrawEntity {
                source,
                player,
                card,
            } => {
                let Some(position) = self
                    .state
                    .player(player)
                    .deck
                    .iter()
                    .position(|candidate| *candidate == card)
                else {
                    return Ok(false);
                };
                let entity = self
                    .state
                    .entity(card)
                    .ok_or(GameError::UnknownEntity(card))?;
                if entity.zone != Zone::Deck || entity.controller != player {
                    return Ok(false);
                }
                self.state.player_mut(player).deck.remove(position);
                self.stage_reserved_draw(player, card, Some(source), queue)?;
                Ok(false)
            }
            EffectSpec::EquipWeapon {
                source: _,
                player,
                card_id,
            } => {
                let definition_kind = self
                    .runtime
                    .definition(&card_id)
                    .ok_or_else(|| GameError::UnknownCard(card_id.clone()))?
                    .kind;
                if definition_kind != CardKind::Weapon {
                    return Err(GameError::CardCannotBeEquipped(card_id));
                }
                let weapon = self.instantiate(&card_id, player, Zone::SetAside)?;
                let equip = self.begin_event(GameEvent::WeaponEquipped { player, weapon })?;
                let before = self.trigger_event(&equip, EventTiming::Before)?;
                queue.push_front(ResolutionItem::CommitEffectWeaponEquip {
                    equip,
                    replacement: None,
                });
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::LoseWeaponDurability {
                source: _,
                weapon,
                amount,
            } => {
                if amount <= 0 {
                    return Ok(false);
                }
                let Some(entity) = self.state.entity(weapon).cloned() else {
                    return Err(GameError::UnknownEntity(weapon));
                };
                let player = entity.controller;
                if entity.kind != CardKind::Weapon
                    || entity.zone != Zone::Weapon
                    || self.state.player(player).weapon != Some(weapon)
                {
                    return Ok(false);
                }
                let broken = {
                    let entity = self.state.entities.get_mut(&weapon).unwrap();
                    entity.damage = entity.damage.saturating_add(amount);
                    entity.health() <= 0
                };
                self.refresh_auras()?;
                if broken {
                    let pending =
                        self.begin_event(GameEvent::WeaponDestroyed { player, weapon })?;
                    let before = self.trigger_event(&pending, EventTiming::Before)?;
                    queue.push_front(ResolutionItem::CommitWeaponDestruction(pending));
                    Self::prepend_effects(queue, before);
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
                stats,
                keywords,
            } => {
                if self.state.player(player).board.len() >= MAX_BOARD_SIZE {
                    return Ok(false);
                }
                let max = self.state.player(player).board.len();
                // Effects may carry a position remembered before an atomic
                // death batch removed several minions. Preserve the intended
                // relative placement by clamping to the current board edge;
                // player-command positions are still validated strictly.
                let position = position.map(|position| position.min(max));
                let definition_kind = self
                    .runtime
                    .definition(&card_id)
                    .ok_or_else(|| GameError::UnknownCard(card_id.clone()))?
                    .kind;
                if definition_kind != CardKind::Minion {
                    return Err(GameError::CardCannotBeSummoned(card_id));
                }
                let entity = self.instantiate(&card_id, player, Zone::SetAside)?;
                if let SummonStats::Base(stats) = &stats {
                    let summoned = self.state.entities.get_mut(&entity).unwrap();
                    summoned.base_attack = stats.attack;
                    summoned.base_health = stats.health.max(1);
                    Self::recompute_entity(summoned);
                }
                if matches!(stats, SummonStats::Final(_)) || !keywords.is_empty() {
                    let id = EnchantmentId(self.state.next_enchantment_id);
                    self.state.next_enchantment_id += 1;
                    let mut modifiers = Vec::new();
                    if let SummonStats::Final(stats) = &stats {
                        modifiers.push(StatModifier {
                            stat: Stat::Attack,
                            operation: ModifierOperation::FinalSet,
                            value: stats.attack,
                        });
                        modifiers.push(StatModifier {
                            stat: Stat::Health,
                            operation: ModifierOperation::FinalSet,
                            value: stats.health,
                        });
                    }
                    let summoned = self.state.entities.get_mut(&entity).unwrap();
                    summoned.enchantments.push(Enchantment {
                        id,
                        source: entity,
                        attack: 0,
                        health: 0,
                        modifiers,
                        keywords,
                        silenciable: true,
                        expires_at: None,
                    });
                    Self::recompute_entity(summoned);
                }
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
            EffectSpec::SummonExisting {
                source: _,
                player,
                card,
                position,
            } => {
                if self.state.player(player).board.len() >= MAX_BOARD_SIZE {
                    return Ok(false);
                }
                let entity = self
                    .state
                    .entity(card)
                    .ok_or(GameError::UnknownEntity(card))?;
                if !matches!(entity.zone, Zone::Graveyard | Zone::Removed)
                    || entity.kind != CardKind::Minion
                {
                    return Ok(false);
                }
                let origin = if entity.zone == Zone::Graveyard {
                    let Some(graveyard_position) = self
                        .state
                        .player(entity.controller)
                        .graveyard
                        .iter()
                        .position(|candidate| *candidate == card)
                    else {
                        return Ok(false);
                    };
                    let origin_player = entity.controller;
                    self.state
                        .player_mut(origin_player)
                        .graveyard
                        .remove(graveyard_position);
                    ReservedSummonOrigin::Graveyard {
                        player: origin_player,
                        position: graveyard_position,
                    }
                } else {
                    ReservedSummonOrigin::Removed {
                        player: entity.controller,
                    }
                };
                self.state.entities.get_mut(&card).unwrap().zone = Zone::SetAside;
                self.refresh_auras()?;
                let pending = self.begin_event(GameEvent::MinionSummoned {
                    player,
                    entity: card,
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
            EffectSpec::SummonCopy {
                source,
                player,
                target,
                position,
                final_stats,
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
                let is_live_template =
                    matches!(template.zone, Zone::Deck | Zone::Hand | Zone::Board);
                if !is_live_template || template.kind != CardKind::Minion {
                    return Ok(false);
                }
                let entity = self.instantiate(&template.card_id, player, Zone::SetAside)?;
                self.copy_card_state(&template, entity);
                if let Some(final_stats) = final_stats {
                    let id = EnchantmentId(self.state.next_enchantment_id);
                    self.state.next_enchantment_id += 1;
                    let modifiers = vec![
                        StatModifier {
                            stat: Stat::Attack,
                            operation: ModifierOperation::FinalSet,
                            value: final_stats.attack,
                        },
                        StatModifier {
                            stat: Stat::Health,
                            operation: ModifierOperation::FinalSet,
                            value: final_stats.health,
                        },
                    ];
                    let copy = self.state.entities.get_mut(&entity).unwrap();
                    copy.damage = 0;
                    copy.enchantments.push(Enchantment {
                        id,
                        source,
                        attack: 0,
                        health: 0,
                        modifiers,
                        keywords: Vec::new(),
                        silenciable: true,
                        expires_at: None,
                    });
                    Self::recompute_entity(copy);
                }
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
                stats,
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
                    stats,
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
            EffectSpec::Destroy { source, targets } => {
                let mut seen = std::collections::BTreeSet::new();
                let mut events = Vec::new();
                let mut weapon_destructions = Vec::new();
                let mut weapon_before = Vec::new();
                for target in targets {
                    if !seen.insert(target) {
                        continue;
                    }
                    let Some(entity) = self.state.entity(target).cloned() else {
                        return Err(GameError::UnknownEntity(target));
                    };
                    if entity.kind == CardKind::Weapon && entity.zone == Zone::Weapon {
                        let pending = self.begin_event(GameEvent::WeaponDestroyed {
                            player: entity.controller,
                            weapon: target,
                        })?;
                        weapon_before.extend(self.trigger_event(&pending, EventTiming::Before)?);
                        weapon_destructions.push(pending);
                        continue;
                    }
                    if entity.zone != Zone::Board
                        || !self.keyword_bool(target, "can_be_destroyed", true, Some(source))?
                    {
                        continue;
                    }
                    match entity.kind {
                        CardKind::Minion => {
                            let target = self.state.entities.get_mut(&target).unwrap();
                            target.damage = entity.max_health.max(1);
                            target.death_source = Some(source);
                        }
                        CardKind::Location => {
                            self.remove_from_zone(target, Zone::Board, entity.controller);
                            self.move_to_graveyard(target, entity.controller);
                            events.push(GameEvent::LocationDestroyed {
                                player: entity.controller,
                                location: target,
                            });
                        }
                        _ => {}
                    }
                }
                self.refresh_auras()?;
                queue.push_front(ResolutionItem::DeathCheck);
                for pending in weapon_destructions.into_iter().rev() {
                    queue.push_front(ResolutionItem::CommitForcedWeaponDestruction(pending));
                }
                for event in events.into_iter().rev() {
                    let triggered = self.publish(event)?;
                    Self::prepend_effects(queue, triggered);
                }
                Self::prepend_effects(queue, weapon_before);
                Ok(false)
            }
            EffectSpec::TriggerHook {
                source: _,
                target,
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
                    .on_continue(&self.state, target, None, &hook, payload.as_ref())
                    .map_err(GameError::Script)?;
                Self::prepend_effects(queue, generated);
                Ok(false)
            }
            EffectSpec::MoveEntity {
                source: _,
                target,
                destination,
                destination_player,
            } => {
                let entity = self
                    .state
                    .entity(target)
                    .cloned()
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
                if destination == ZonePlacement::Secret {
                    let is_secret = entity.kind == CardKind::Spell
                        && self.keyword_bool(target, "enters_secret_zone", false, None)?;
                    if !is_secret
                        || self.state.player(entity.owner).secrets.len() >= MAX_SECRET_SIZE
                    {
                        return Ok(false);
                    }
                }
                if destination == ZonePlacement::Board
                    && (entity.zone != Zone::Graveyard
                        || entity.kind != CardKind::Minion
                        || self.state.player(entity.owner).board.len() >= MAX_BOARD_SIZE)
                {
                    return Ok(false);
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
                    destination_player,
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
                let can_take_minion = entity.zone == Zone::Board
                    && entity.kind == CardKind::Minion
                    && self.state.player(player).board.len() < MAX_BOARD_SIZE;
                let can_take_secret = entity.zone == Zone::Secret
                    && self.state.player(player).secrets.len() < MAX_SECRET_SIZE;
                if entity.controller == player || (!can_take_minion && !can_take_secret) {
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
            EffectSpec::ChangeControllerUntilEndOfTurn {
                source,
                target,
                player,
            } => {
                let entity = self
                    .state
                    .entity(target)
                    .ok_or(GameError::UnknownEntity(target))?;
                let can_take = entity.zone == Zone::Board
                    && entity.kind == CardKind::Minion
                    && self.state.player(player).board.len() < MAX_BOARD_SIZE;
                if entity.controller == player || !can_take {
                    return Ok(false);
                }
                let pending = self.begin_event(GameEvent::ControllerChanged {
                    source,
                    entity: target,
                    from: entity.controller,
                    to: player,
                })?;
                let before = self.trigger_event(&pending, EventTiming::Before)?;
                queue.push_front(ResolutionItem::CommitTemporaryControllerChange {
                    change: pending,
                    expires_at_turn: self.state.turn,
                });
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::ForceAttack {
                source: _,
                attacker,
                defender,
            } => {
                let valid = self.state.entity(attacker).is_some_and(|entity| {
                    entity.zone == Zone::Board
                        && entity.kind == CardKind::Minion
                        && entity.health() > 0
                }) && self.state.entity(defender).is_some_and(|entity| {
                    matches!(entity.zone, Zone::Board | Zone::Hero)
                        && matches!(entity.kind, CardKind::Minion | CardKind::Hero)
                        && entity.health() > 0
                }) && self.state.entities[&attacker].controller
                    != self.state.entities[&defender].controller;
                if !valid {
                    return Ok(false);
                }
                let pending = self.begin_event(GameEvent::Attack {
                    attacker,
                    defender,
                    collateral: Vec::new(),
                })?;
                let before = self.trigger_event(&pending, EventTiming::Before)?;
                queue.push_front(ResolutionItem::CommitEvent(pending));
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::Transform {
                source,
                transforms,
                preserve_attached_scripts,
            } => {
                let mut seen = std::collections::BTreeSet::new();
                let mut pending_events = Vec::new();
                let mut before = Vec::new();
                for (target, card_id) in transforms {
                    if !seen.insert(target) {
                        continue;
                    }
                    let Some(entity) = self.state.entity(target) else {
                        continue;
                    };
                    if matches!(
                        entity.zone,
                        Zone::Hero | Zone::HeroPower | Zone::SetAside | Zone::Removed
                    ) || entity.card_id == card_id
                    {
                        continue;
                    }
                    let definition_kind = self
                        .runtime
                        .definition(&card_id)
                        .ok_or_else(|| GameError::UnknownCard(card_id.clone()))?
                        .kind;
                    let hidden_zone_can_change_kind =
                        matches!(entity.zone, Zone::Hand | Zone::Deck)
                            && matches!(
                                definition_kind,
                                CardKind::Hero
                                    | CardKind::Minion
                                    | CardKind::Spell
                                    | CardKind::Weapon
                                    | CardKind::Location
                            );
                    if definition_kind != entity.kind && !hidden_zone_can_change_kind {
                        return Err(GameError::CardCannotTransformInto(card_id));
                    }
                    let pending = self.begin_event(GameEvent::Transformed {
                        source,
                        entity: target,
                        from_card: entity.card_id.clone(),
                        to_card: card_id,
                    })?;
                    before.extend(self.trigger_event(&pending, EventTiming::Before)?);
                    pending_events.push(pending);
                }
                if pending_events.is_empty() {
                    return Ok(false);
                }
                queue.push_front(ResolutionItem::CommitTransformGroup {
                    transforms: pending_events,
                    preserve_attached_scripts,
                });
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::TransformIntoCopy {
                source,
                target,
                template,
                final_stats,
                preserve_attached_scripts,
            } => {
                let Some(entity) = self.state.entity(target) else {
                    return Ok(false);
                };
                let template = self
                    .state
                    .entity(template)
                    .cloned()
                    .ok_or(GameError::UnknownEntity(template))?;
                if matches!(
                    entity.zone,
                    Zone::Hero | Zone::HeroPower | Zone::SetAside | Zone::Removed
                ) || entity.kind != template.kind
                {
                    return Ok(false);
                }
                let pending = self.begin_event(GameEvent::Transformed {
                    source,
                    entity: target,
                    from_card: entity.card_id.clone(),
                    to_card: template.card_id.clone(),
                })?;
                let before = self.trigger_event(&pending, EventTiming::Before)?;
                queue.push_front(ResolutionItem::CommitTransformIntoCopy {
                    transform: pending,
                    template,
                    final_stats,
                    preserve_attached_scripts,
                });
                Self::prepend_effects(queue, before);
                Ok(false)
            }
            EffectSpec::Continue {
                source,
                hook,
                payload,
                continuation_owner,
            } => {
                if hook.is_empty() || hook.len() > 64 {
                    return Err(GameError::InvalidContinuationHook);
                }
                if let Some(payload) = &payload {
                    payload.validate().map_err(GameError::InvalidChoiceValue)?;
                }
                let generated = self
                    .runtime
                    .on_continue(
                        &self.state,
                        source,
                        continuation_owner.as_deref(),
                        &hook,
                        payload.as_ref(),
                    )
                    .map_err(GameError::Script)?;
                Self::prepend_effects(queue, generated);
                Ok(false)
            }
            EffectSpec::SpendPlayerResourceAndContinue {
                source,
                player,
                resource,
                minimum,
                maximum,
                hook,
                continuation_owner,
            } => {
                if hook.is_empty() || hook.len() > 64 {
                    return Err(GameError::InvalidContinuationHook);
                }
                if resource.is_empty() || resource.len() > 64 {
                    return Err(GameError::InvalidPlayerResource);
                }
                if minimum > maximum {
                    return Err(GameError::InvalidPlayerResourceSpend);
                }
                let available = self.state.player(player).resource(&resource);
                let candidate = available.min(maximum);
                let amount = if candidate >= minimum { candidate } else { 0 };
                Self::prepend_effects(
                    queue,
                    vec![EffectSpec::Continue {
                        source,
                        hook,
                        payload: Some(ChoiceValue::Integer(i64::from(amount))),
                        continuation_owner,
                    }],
                );
                if amount == 0 {
                    return Ok(false);
                }
                let state = self.state.player_mut(player);
                *state.resources.get_mut(&resource).unwrap() -= amount;
                let spent = state.resources_spent.entry(resource.clone()).or_default();
                *spent = spent.saturating_add(amount);
                queue.push_front(ResolutionItem::DeathCheck);
                let triggered = self.publish(GameEvent::PlayerResourceSpent {
                    source,
                    player,
                    resource,
                    amount,
                })?;
                Self::prepend_effects(queue, triggered);
                Ok(false)
            }
            EffectSpec::CancelEvent { source: _, event } => {
                let pending = Self::find_pending_event_mut(queue, event)
                    .ok_or(GameError::EventNotPending(event))?;
                pending.cancelled = true;
                Ok(false)
            }
            EffectSpec::ModifyEventAmount {
                source: _,
                event,
                operation,
                value,
            } => {
                let pending = Self::find_pending_event_mut(queue, event)
                    .ok_or(GameError::EventNotPending(event))?;
                match &mut pending.event {
                    GameEvent::Damaged { amount, .. } | GameEvent::Healed { amount, .. } => {
                        *amount = match operation {
                            ModifierOperation::Set | ModifierOperation::FinalSet => value,
                            ModifierOperation::Add | ModifierOperation::PreFinalAdd => {
                                amount.saturating_add(value)
                            }
                            ModifierOperation::Multiply => amount.saturating_mul(value),
                        }
                        .max(0);
                    }
                    GameEvent::Fatigue { amount, .. } => {
                        let current = i64::from(*amount);
                        let modified = match operation {
                            ModifierOperation::Set | ModifierOperation::FinalSet => {
                                i64::from(value)
                            }
                            ModifierOperation::Add | ModifierOperation::PreFinalAdd => {
                                current.saturating_add(i64::from(value))
                            }
                            ModifierOperation::Multiply => current.saturating_mul(i64::from(value)),
                        };
                        *amount = modified.clamp(0, i64::from(u32::MAX)) as u32;
                    }
                    _ => return Err(GameError::EventAmountNotReplaceable(event)),
                }
                Ok(false)
            }
            EffectSpec::SetAttackDefender {
                source: _,
                event,
                defender,
            } => {
                let pending = Self::find_pending_event_mut(queue, event)
                    .ok_or(GameError::EventNotPending(event))?;
                match &mut pending.event {
                    GameEvent::Attack {
                        defender: selected, ..
                    } => *selected = defender,
                    _ => return Err(GameError::EventAttackNotReplaceable(event)),
                }
                Ok(false)
            }
            EffectSpec::AddAttackCollateral {
                source: _,
                event,
                targets,
                amount,
            } => {
                let pending = Self::find_pending_event_mut(queue, event)
                    .ok_or(GameError::EventNotPending(event))?;
                match &mut pending.event {
                    GameEvent::Attack { collateral, .. } => {
                        for target in targets {
                            if !collateral.iter().any(|(existing, _)| *existing == target) {
                                collateral.push((target, amount.max(0)));
                            }
                        }
                    }
                    _ => return Err(GameError::EventAttackNotReplaceable(event)),
                }
                Ok(false)
            }
            EffectSpec::SetDamageTarget {
                source: _,
                event,
                target,
            } => {
                let entity = self
                    .state
                    .entity(target)
                    .ok_or(GameError::UnknownEntity(target))?;
                if !matches!(entity.kind, CardKind::Hero | CardKind::Minion) {
                    return Err(GameError::InvalidTarget(target));
                }
                let pending = Self::find_pending_event_mut(queue, event)
                    .ok_or(GameError::EventNotPending(event))?;
                match &mut pending.event {
                    GameEvent::Damaged {
                        target: selected, ..
                    } => *selected = target,
                    _ => return Err(GameError::EventDamageNotReplaceable(event)),
                }
                Ok(false)
            }
            EffectSpec::SetSpellTarget {
                source: _,
                event,
                target,
            } => {
                let entity = self
                    .state
                    .entity(target)
                    .ok_or(GameError::UnknownEntity(target))?;
                if entity.kind != CardKind::Minion || entity.zone != Zone::Board {
                    return Err(GameError::InvalidTarget(target));
                }
                let pending = Self::find_pending_event_mut(queue, event)
                    .ok_or(GameError::EventNotPending(event))?;
                match &mut pending.event {
                    GameEvent::SpellTargeted {
                        target: selected, ..
                    } => *selected = target,
                    _ => return Err(GameError::EventSpellTargetNotReplaceable(event)),
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
