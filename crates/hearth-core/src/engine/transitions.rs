use super::*;

impl<R: CardRuntime> Game<R> {
    pub(super) fn prepare_combat(
        &mut self,
        attack_id: EventId,
        attacker: EntityId,
        defender: EntityId,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        if self.state.outcome.is_some() {
            return Ok(());
        }
        let Some(attacker_entity) = self.state.entity(attacker).cloned() else {
            return Ok(());
        };
        let Some(defender_entity) = self.state.entity(defender).cloned() else {
            return Ok(());
        };
        if !matches!(attacker_entity.zone, Zone::Board | Zone::Hero)
            || !matches!(defender_entity.zone, Zone::Board | Zone::Hero)
            || attacker_entity.health() <= 0
            || defender_entity.health() <= 0
        {
            return Ok(());
        }

        let mut damage = vec![self.begin_event(GameEvent::Damaged {
            source: attacker,
            target: defender,
            amount: attacker_entity.attack.max(0),
        })?];
        if defender_entity.kind == CardKind::Minion && defender_entity.attack > 0 {
            damage.push(self.begin_event(GameEvent::Damaged {
                source: defender,
                target: attacker,
                amount: defender_entity.attack,
            })?);
        }
        let mut before = Vec::new();
        for event in &damage {
            before.extend(self.trigger_event(event, EventTiming::Before)?);
        }
        queue.push_front(ResolutionItem::CommitCombat {
            attack: PendingEvent {
                id: attack_id,
                event: GameEvent::Attack { attacker, defender },
                cancelled: false,
            },
            damage,
        });
        Self::prepend_effects(queue, before);
        Ok(())
    }

    pub(super) fn commit_combat(
        &mut self,
        attack: PendingEvent,
        damage: Vec<PendingEvent>,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        if attack.cancelled {
            return Ok(());
        }
        let GameEvent::Attack { attacker, defender } = attack.event else {
            unreachable!("combat item must contain an attack event")
        };
        let valid = self.state.entity(attacker).is_some_and(|entity| {
            matches!(entity.zone, Zone::Board | Zone::Hero)
                && matches!(entity.kind, CardKind::Minion | CardKind::Hero)
                && entity.health() > 0
        }) && self.state.entity(defender).is_some_and(|entity| {
            matches!(entity.zone, Zone::Board | Zone::Hero)
                && matches!(entity.kind, CardKind::Minion | CardKind::Hero)
                && entity.health() > 0
        });
        if !valid {
            return Ok(());
        }

        let mut notifications = Vec::new();
        for pending in damage {
            if pending.cancelled {
                continue;
            }
            let GameEvent::Damaged {
                source,
                target,
                amount,
            } = pending.event
            else {
                unreachable!("combat damage item must contain a damage event")
            };
            let event = self.apply_damage(source, target, amount)?;
            notifications.push((pending.id, event));
        }
        let player = self.state.entities[&attacker].controller;
        let mut staged_weapon_destruction = None;
        if self.state.entities[&attacker].kind == CardKind::Hero
            && let Some(weapon) = self.state.player(player).weapon
        {
            let broken = {
                let weapon_entity = self.state.entities.get_mut(&weapon).unwrap();
                weapon_entity.damage += 1;
                weapon_entity.health() <= 0
            };
            if broken {
                let pending = self.begin_event(GameEvent::WeaponDestroyed { player, weapon })?;
                let before = self.trigger_event(&pending, EventTiming::Before)?;
                staged_weapon_destruction = Some((pending, before));
            }
        }
        self.refresh_auras()?;
        notifications.push((attack.id, GameEvent::Attack { attacker, defender }));
        queue.push_front(ResolutionItem::DeathCheck);
        if let Some((destruction, before)) = staged_weapon_destruction {
            queue.push_front(ResolutionItem::CommitWeaponDestruction(destruction));
            Self::prepend_effects(queue, before);
        }
        queue.push_front(ResolutionItem::PublishAfterGroup {
            events: notifications,
        });
        Ok(())
    }

    pub(super) fn commit_weapon_destruction(
        &mut self,
        destruction: PendingEvent,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let GameEvent::WeaponDestroyed { player, weapon } = destruction.event else {
            unreachable!("weapon destruction item must contain weapon_destroyed")
        };
        if self.state.player(player).weapon != Some(weapon) {
            return Ok(());
        }
        let current_health = self.state.entities[&weapon].health();
        if destruction.cancelled {
            if current_health <= 0 {
                let entity = self.state.entities.get_mut(&weapon).unwrap();
                entity.damage = (entity.max_health - 1).max(0);
            }
            self.refresh_auras()?;
            return Ok(());
        }
        if current_health > 0 {
            return Ok(());
        }
        self.destroy_weapon(player, weapon);
        self.refresh_auras()?;
        queue.push_front(ResolutionItem::PublishAfter {
            id: destruction.id,
            event: GameEvent::WeaponDestroyed { player, weapon },
        });
        Ok(())
    }

    pub(super) fn commit_damage_group(
        &mut self,
        damage: Vec<PendingEvent>,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let mut notifications = Vec::new();
        for pending in damage {
            if pending.cancelled {
                continue;
            }
            let GameEvent::Damaged {
                source,
                target,
                amount,
            } = pending.event
            else {
                unreachable!("damage group must contain damage events")
            };
            let event = self.apply_damage(source, target, amount)?;
            notifications.push((pending.id, event));
        }
        self.refresh_auras()?;
        queue.push_front(ResolutionItem::DeathCheck);
        queue.push_front(ResolutionItem::PublishAfterGroup {
            events: notifications,
        });
        Ok(())
    }

    pub(super) fn commit_zone_change(
        &mut self,
        change: PendingEvent,
        destination: ZonePlacement,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let GameEvent::ZoneChanged {
            entity,
            from,
            to: requested_to,
        } = change.event
        else {
            unreachable!("zone change item must contain a zone_changed event")
        };
        if change.cancelled {
            return Ok(());
        }
        let Some(current) = self.state.entity(entity).cloned() else {
            return Err(GameError::UnknownEntity(entity));
        };
        // A nested before effect may already have moved the entity. The older movement then
        // becomes stale rather than pulling it out of its new zone.
        if current.zone != from {
            return Ok(());
        }

        let source_controller = current.controller;
        let destination_player = current.owner;
        self.remove_from_zone(entity, from, source_controller);

        let actual_to = match destination {
            ZonePlacement::Hand
                if self.state.player(destination_player).hand.len() >= MAX_HAND_SIZE =>
            {
                self.move_to_graveyard(entity, destination_player);
                Zone::Graveyard
            }
            ZonePlacement::Hand => {
                self.reset_after_hidden_zone_change(entity, destination_player);
                self.state.entities.get_mut(&entity).unwrap().zone = Zone::Hand;
                self.state
                    .entities
                    .get_mut(&entity)
                    .unwrap()
                    .entered_hand_turn = Some(self.state.turn);
                self.state.player_mut(destination_player).hand.push(entity);
                Zone::Hand
            }
            ZonePlacement::DeckTop | ZonePlacement::DeckBottom | ZonePlacement::DeckRandom => {
                self.reset_after_hidden_zone_change(entity, destination_player);
                self.state.entities.get_mut(&entity).unwrap().zone = Zone::Deck;
                let deck_len = self.state.player(destination_player).deck.len();
                let position = match destination {
                    ZonePlacement::DeckTop => 0,
                    ZonePlacement::DeckBottom => deck_len,
                    ZonePlacement::DeckRandom => {
                        self.state.random_counter += 1;
                        self.rng.random_range(0..=deck_len)
                    }
                    _ => unreachable!(),
                };
                self.state
                    .player_mut(destination_player)
                    .deck
                    .insert(position, entity);
                Zone::Deck
            }
            ZonePlacement::Graveyard => {
                self.move_to_graveyard(entity, destination_player);
                Zone::Graveyard
            }
            ZonePlacement::Removed => {
                self.state.entities.get_mut(&entity).unwrap().zone = Zone::Removed;
                Zone::Removed
            }
        };
        self.refresh_auras()?;
        let event = GameEvent::ZoneChanged {
            entity,
            from,
            to: actual_to,
        };
        queue.push_front(ResolutionItem::DeathCheck);
        queue.push_front(ResolutionItem::PublishAfter {
            id: change.id,
            event,
        });

        debug_assert_eq!(requested_to, destination.zone());
        Ok(())
    }

    pub(super) fn commit_controller_change(
        &mut self,
        change: PendingEvent,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        if change.cancelled {
            return Ok(());
        }
        let GameEvent::ControllerChanged {
            source,
            entity,
            from,
            to,
        } = change.event
        else {
            unreachable!("controller change item must contain a controller_changed event")
        };
        let still_valid = self.state.entity(entity).is_some_and(|candidate| {
            candidate.zone == Zone::Board
                && candidate.kind == CardKind::Minion
                && candidate.controller == from
        });
        if !still_valid || self.state.player(to).board.len() >= MAX_BOARD_SIZE {
            return Ok(());
        }

        self.state
            .player_mut(from)
            .board
            .retain(|candidate| *candidate != entity);
        self.state.player_mut(to).board.push(entity);
        let entity_state = self.state.entities.get_mut(&entity).unwrap();
        entity_state.controller = to;
        entity_state.attacks_this_turn = 0;
        entity_state.exhausted = true;
        let ready = self.keyword_bool(entity, "ready_on_summon", false, None)?;
        self.state.entities.get_mut(&entity).unwrap().exhausted = !ready;
        self.refresh_auras()?;
        queue.push_front(ResolutionItem::DeathCheck);
        queue.push_front(ResolutionItem::PublishAfter {
            id: change.id,
            event: GameEvent::ControllerChanged {
                source,
                entity,
                from,
                to,
            },
        });
        Ok(())
    }

    pub(super) fn commit_transform(
        &mut self,
        transform: PendingEvent,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        if transform.cancelled {
            return Ok(());
        }
        let GameEvent::Transformed {
            source,
            entity,
            from_card,
            to_card,
        } = transform.event
        else {
            unreachable!("transform item must contain a transformed event")
        };
        let Some(current) = self.state.entity(entity) else {
            return Ok(());
        };
        let current_zone = current.zone;
        let current_kind = current.kind;
        let still_valid = !matches!(
            current_zone,
            Zone::Hero | Zone::HeroPower | Zone::SetAside | Zone::Removed
        ) && current.card_id == from_card;
        if !still_valid {
            return Ok(());
        }
        let definition = self
            .runtime
            .definition(&to_card)
            .cloned()
            .ok_or_else(|| GameError::UnknownCard(to_card.clone()))?;
        let hand_can_change_kind = current_zone == Zone::Hand
            && matches!(
                definition.kind,
                CardKind::Hero
                    | CardKind::Minion
                    | CardKind::Spell
                    | CardKind::Weapon
                    | CardKind::Location
            );
        if definition.kind != current_kind && !hand_can_change_kind {
            return Err(GameError::CardCannotTransformInto(to_card));
        }

        let entity_state = self.state.entities.get_mut(&entity).unwrap();
        entity_state.card_id = definition.id.clone();
        entity_state.name = definition.name;
        entity_state.kind = definition.kind;
        entity_state.base_attack = definition.attack;
        entity_state.base_health = definition.health;
        entity_state.base_cost = definition.cost;
        entity_state.base_spell_damage = 0;
        entity_state.base_keywords = definition.keywords.clone();
        entity_state.attack = definition.attack;
        entity_state.max_health = definition.health;
        entity_state.damage = 0;
        entity_state.armor = 0;
        entity_state.cost = definition.cost;
        entity_state.spell_damage = 0;
        entity_state.frozen = false;
        entity_state.frozen_since_turn = None;
        entity_state.keywords = definition.keywords;
        entity_state.disabled_keywords.clear();
        entity_state.aura_attack = 0;
        entity_state.aura_health = 0;
        entity_state.aura_cost = 0;
        entity_state.aura_spell_damage = 0;
        entity_state.aura_keywords.clear();
        entity_state.enchantments.clear();
        entity_state.silenced = false;
        entity_state.script_data.clear();
        entity_state.attached_cards.clear();
        Self::recompute_entity(entity_state);
        self.refresh_auras()?;
        if current_zone == Zone::Board {
            queue.push_front(ResolutionItem::DeathCheck);
        }
        queue.push_front(ResolutionItem::PublishAfter {
            id: transform.id,
            event: GameEvent::Transformed {
                source,
                entity,
                from_card,
                to_card: definition.id,
            },
        });
        Ok(())
    }

    pub(super) fn reset_after_hidden_zone_change(&mut self, entity: EntityId, player: PlayerId) {
        let timestamp = self.state.next_timestamp;
        self.state.next_timestamp += 1;
        let entity = self.state.entities.get_mut(&entity).unwrap();
        entity.controller = player;
        entity.damage = 0;
        entity.armor = 0;
        entity.exhausted = false;
        entity.frozen = false;
        entity.frozen_since_turn = None;
        entity.attacks_this_turn = 0;
        entity.location_cooldown = 0;
        entity.timestamp = timestamp;
        entity.enchantments.clear();
        entity.silenced = false;
        entity.disabled_keywords.clear();
        entity.cards_played_before = 0;
        entity.script_data.clear();
        entity.attached_cards.clear();
        Self::recompute_entity(entity);
    }

    pub(super) fn find_pending_event_mut(
        queue: &mut VecDeque<ResolutionItem>,
        id: EventId,
    ) -> Option<&mut PendingEvent> {
        for item in queue {
            match item {
                ResolutionItem::CommitEvent(event) if event.id == id => return Some(event),
                ResolutionItem::CommitFatigue(event) if event.id == id => return Some(event),
                ResolutionItem::CommitCardPlay { play, .. } if play.id == id => return Some(play),
                ResolutionItem::CommitDiscard(event) if event.id == id => return Some(event),
                ResolutionItem::CommitTradeDraw(event) if event.id == id => return Some(event),
                ResolutionItem::CommitHeroPower { use_event, .. } if use_event.id == id => {
                    return Some(use_event);
                }
                ResolutionItem::CommitLocationUse(event) if event.id == id => return Some(event),
                ResolutionItem::CommitWeaponEquip {
                    equip, replacement, ..
                } => {
                    if equip.id == id {
                        return Some(equip);
                    }
                    if let Some(event) = replacement
                        && event.id == id
                    {
                        return Some(event);
                    }
                }
                ResolutionItem::CommitWeaponDestruction(event) if event.id == id => {
                    return Some(event);
                }
                ResolutionItem::CommitCombat { attack, damage } => {
                    if attack.id == id {
                        return Some(attack);
                    }
                    if let Some(event) = damage.iter_mut().find(|event| event.id == id) {
                        return Some(event);
                    }
                }
                ResolutionItem::CommitDamageGroup { damage } => {
                    if let Some(event) = damage.iter_mut().find(|event| event.id == id) {
                        return Some(event);
                    }
                }
                ResolutionItem::CommitSummon { summon, .. } if summon.id == id => {
                    return Some(summon);
                }
                ResolutionItem::CommitZoneChange { change, .. } if change.id == id => {
                    return Some(change);
                }
                ResolutionItem::CommitControllerChange(change) if change.id == id => {
                    return Some(change);
                }
                ResolutionItem::CommitTransform(transform) if transform.id == id => {
                    return Some(transform);
                }
                _ => {}
            }
        }
        None
    }

    pub(super) fn prepend_effects(queue: &mut VecDeque<ResolutionItem>, effects: Vec<EffectSpec>) {
        for effect in effects.into_iter().rev() {
            queue.push_front(ResolutionItem::Effect(effect));
        }
    }
}
