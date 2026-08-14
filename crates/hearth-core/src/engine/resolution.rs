use super::*;

impl<R: CardRuntime> Game<R> {
    pub(super) fn resolve_effects(&mut self, effects: Vec<EffectSpec>) -> Result<(), GameError> {
        self.resolve_items(effects.into_iter().map(ResolutionItem::Effect).collect())
    }

    pub(super) fn resolve_items(&mut self, items: Vec<ResolutionItem>) -> Result<(), GameError> {
        let mut queue: VecDeque<_> = items.into();
        let mut steps = 0;
        loop {
            while let Some(item) = queue.pop_front() {
                steps += 1;
                if steps > MAX_RESOLUTION_STEPS {
                    return Err(GameError::ResolutionLimit);
                }
                match item {
                    ResolutionItem::Effect(effect) => {
                        if self.resolve_effect_item(effect, &mut queue)? {
                            return Ok(());
                        }
                    }
                    ResolutionItem::DrawOne { player } => {
                        self.stage_draw(player, &mut queue)?;
                    }
                    ResolutionItem::CommitFatigue(event) => {
                        self.commit_fatigue(event, &mut queue)?;
                    }
                    ResolutionItem::CommitEvent(event) => {
                        self.commit_event(event, &mut queue)?;
                    }
                    ResolutionItem::CommitCardPlay {
                        play,
                        target,
                        position,
                    } => {
                        self.commit_card_play(play, target, position, &mut queue)?;
                    }
                    ResolutionItem::CommitDiscard(event) => {
                        self.commit_discard(event, &mut queue)?;
                    }
                    ResolutionItem::CommitTradeDraw(event) => {
                        self.commit_trade_draw(event, &mut queue)?;
                    }
                    ResolutionItem::CompleteTrade { player, card } => {
                        self.complete_trade(player, card, &mut queue)?;
                    }
                    ResolutionItem::CommitHeroPower { use_event, target } => {
                        self.commit_hero_power(use_event, target, &mut queue)?;
                    }
                    ResolutionItem::CommitLocationUse(event) => {
                        self.commit_location_use(event, &mut queue)?;
                    }
                    ResolutionItem::DestroySpentLocation { player, location } => {
                        self.destroy_spent_location(player, location, &mut queue)?;
                    }
                    ResolutionItem::CommitWeaponEquip {
                        equip,
                        card_play_id,
                        target,
                        replacement,
                    } => {
                        self.commit_weapon_equip(
                            equip,
                            card_play_id,
                            target,
                            replacement,
                            &mut queue,
                        )?;
                    }
                    ResolutionItem::CommitWeaponDestruction(event) => {
                        self.commit_weapon_destruction(event, &mut queue)?;
                    }
                    ResolutionItem::CommitCombat { attack, damage } => {
                        self.commit_combat(attack, damage, &mut queue)?;
                    }
                    ResolutionItem::CommitDamageGroup { damage } => {
                        self.commit_damage_group(damage, &mut queue)?;
                    }
                    ResolutionItem::CommitSummon {
                        summon,
                        position,
                        origin,
                    } => {
                        if summon.cancelled {
                            match origin {
                                ReservedSummonOrigin::Generated => {
                                    self.cancel_pending_event(summon)?;
                                }
                                origin @ ReservedSummonOrigin::Deck { .. } => {
                                    let GameEvent::MinionSummoned { entity, .. } = summon.event
                                    else {
                                        unreachable!(
                                            "summon item must contain a minion_summoned event"
                                        );
                                    };
                                    self.restore_reserved_recruit(summon.id, entity, &origin)?;
                                    self.refresh_auras()?;
                                }
                            }
                        } else if let GameEvent::MinionSummoned { player, entity } = summon.event {
                            self.commit_reserved_summon(
                                summon.id, player, entity, position, origin, &mut queue,
                            )?;
                        } else {
                            unreachable!("summon item must contain a minion_summoned event");
                        }
                    }
                    ResolutionItem::CommitZoneChange {
                        change,
                        destination,
                    } => {
                        self.commit_zone_change(change, destination, &mut queue)?;
                    }
                    ResolutionItem::CommitControllerChange(change) => {
                        self.commit_controller_change(change, &mut queue)?;
                    }
                    ResolutionItem::CommitTransform(transform) => {
                        self.commit_transform(transform, &mut queue)?;
                    }
                    ResolutionItem::SummonFreshCopy {
                        player,
                        card_id,
                        position,
                        health,
                        without_keywords,
                    } => {
                        self.stage_fresh_copy(
                            player,
                            &card_id,
                            position,
                            health,
                            &without_keywords,
                            &mut queue,
                        )?;
                    }
                    ResolutionItem::PublishAfter { id, event } => {
                        let triggered = self.publish_after(id, event)?;
                        Self::prepend_effects(&mut queue, triggered);
                    }
                    ResolutionItem::PublishAfterGroup { events } => {
                        let triggered = self.publish_after_group(events)?;
                        Self::prepend_effects(&mut queue, triggered);
                    }
                    ResolutionItem::DeathCheck => {
                        self.run_death_check(&mut queue)?;
                        if self.state.outcome.is_some() {
                            self.abandon_resolution_queue(&mut queue)?;
                        }
                    }
                }
            }

            self.run_death_check(&mut queue)?;
            if queue.is_empty() {
                break;
            }
        }
        Ok(())
    }

    pub(super) fn abandon_resolution_queue(
        &mut self,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        while let Some(item) = queue.pop_front() {
            match item {
                ResolutionItem::CommitCardPlay { play, .. } => {
                    if let GameEvent::CardPlayed { player, card } = play.event
                        && self
                            .state
                            .entity(card)
                            .is_some_and(|entity| entity.zone == Zone::SetAside)
                    {
                        self.move_to_graveyard(card, player);
                    }
                }
                ResolutionItem::CommitWeaponEquip { equip, .. } => {
                    if let GameEvent::WeaponEquipped { player, weapon } = equip.event
                        && self
                            .state
                            .entity(weapon)
                            .is_some_and(|entity| entity.zone == Zone::SetAside)
                    {
                        self.move_to_graveyard(weapon, player);
                    }
                }
                ResolutionItem::CompleteTrade { player, card }
                    if self
                        .state
                        .entity(card)
                        .is_some_and(|entity| entity.zone == Zone::SetAside) =>
                {
                    let position = self
                        .rng
                        .random_range(0..=self.state.player(player).deck.len());
                    self.state.random_counter = self.state.random_counter.saturating_add(1);
                    self.state.entities.get_mut(&card).unwrap().zone = Zone::Deck;
                    self.state.player_mut(player).deck.insert(position, card);
                }
                ResolutionItem::CommitWeaponDestruction(event) => {
                    if let GameEvent::WeaponDestroyed { player, weapon } = event.event
                        && self.state.player(player).weapon == Some(weapon)
                        && self.state.entities[&weapon].health() <= 0
                    {
                        self.destroy_weapon(player, weapon);
                    }
                }
                ResolutionItem::CommitEvent(event) => match event.event {
                    GameEvent::CardDrawn { player, card }
                    | GameEvent::CardBurned { player, card }
                        if self
                            .state
                            .entity(card)
                            .is_some_and(|entity| entity.zone == Zone::SetAside) =>
                    {
                        self.state.entities.get_mut(&card).unwrap().zone = Zone::Deck;
                        self.state.player_mut(player).deck.push_front(card);
                    }
                    GameEvent::MinionSummoned { entity, .. }
                        if self
                            .state
                            .entity(entity)
                            .is_some_and(|entity| entity.zone == Zone::SetAside) =>
                    {
                        self.state.entities.get_mut(&entity).unwrap().zone = Zone::Removed;
                    }
                    _ => {}
                },
                ResolutionItem::CommitSummon { summon, origin, .. }
                    if matches!(summon.event, GameEvent::MinionSummoned { .. }) =>
                {
                    if let GameEvent::MinionSummoned { entity, .. } = summon.event
                        && self
                            .state
                            .entity(entity)
                            .is_some_and(|entity| entity.zone == Zone::SetAside)
                    {
                        match origin {
                            ReservedSummonOrigin::Generated => {
                                self.state.entities.get_mut(&entity).unwrap().zone = Zone::Removed;
                            }
                            origin @ ReservedSummonOrigin::Deck { .. } => {
                                self.restore_reserved_recruit(summon.id, entity, &origin)?;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        self.refresh_auras()?;
        Ok(())
    }
}
