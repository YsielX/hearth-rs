use super::*;

impl<R: CardRuntime> Game<R> {
    pub(super) fn begin_event(&mut self, event: GameEvent) -> Result<PendingEvent, GameError> {
        let id = EventId(self.state.next_event_id);
        self.state.next_event_id += 1;
        Ok(PendingEvent {
            id,
            event,
            cancelled: false,
        })
    }

    pub(super) fn trigger_event(
        &mut self,
        pending: &PendingEvent,
        timing: EventTiming,
    ) -> Result<Vec<EffectSpec>, GameError> {
        let event = ScriptEvent {
            id: pending.id,
            timing,
            event: pending.event.clone(),
        };
        self.collect_triggers(&event)
    }

    pub(super) fn commit_event(
        &mut self,
        pending: PendingEvent,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        if pending.cancelled {
            return self.cancel_pending_event(pending);
        }
        match pending.event {
            GameEvent::Attack {
                attacker,
                defender,
                collateral,
            } => self.prepare_combat(pending.id, attacker, defender, collateral, queue),
            GameEvent::Damaged {
                source,
                target,
                amount,
            } => {
                let event = self.apply_damage(source, target, amount)?;
                let notifications = vec![(pending.id, event)];
                self.refresh_auras()?;
                queue.push_front(ResolutionItem::DeathCheck);
                queue.push_front(ResolutionItem::PublishAfterGroup {
                    events: notifications,
                });
                Ok(())
            }
            GameEvent::Healed {
                source,
                target,
                amount,
            } => {
                let entity = self
                    .state
                    .entities
                    .get_mut(&target)
                    .ok_or(GameError::UnknownEntity(target))?;
                let actual = if entity.kind == CardKind::Location {
                    0
                } else {
                    amount.max(0).min(entity.damage)
                };
                entity.damage -= actual;
                self.refresh_auras()?;
                queue.push_front(ResolutionItem::PublishAfter {
                    id: pending.id,
                    event: GameEvent::Healed {
                        source,
                        target,
                        amount: actual,
                    },
                });
                Ok(())
            }
            GameEvent::CardDrawn {
                player,
                card,
                source,
            } => self.commit_reserved_draw(pending.id, player, card, source, false, queue),
            GameEvent::CardBurned {
                player,
                card,
                source,
            } => self.commit_reserved_draw(pending.id, player, card, source, true, queue),
            GameEvent::MinionSummoned { player, entity } => self.commit_reserved_summon(
                pending.id,
                player,
                entity,
                None,
                ReservedSummonOrigin::Generated,
                queue,
            ),
            event => {
                queue.push_front(ResolutionItem::PublishAfter {
                    id: pending.id,
                    event,
                });
                Ok(())
            }
        }
    }

    pub(super) fn cancel_pending_event(&mut self, pending: PendingEvent) -> Result<(), GameError> {
        match pending.event {
            GameEvent::CardDrawn { player, card, .. }
            | GameEvent::CardBurned { player, card, .. } => {
                let entity = self
                    .state
                    .entities
                    .get_mut(&card)
                    .ok_or(GameError::UnknownEntity(card))?;
                if entity.zone != Zone::SetAside {
                    return Err(GameError::InvalidReservedEntity(pending.id));
                }
                entity.zone = Zone::Deck;
                self.state.player_mut(player).deck.push_front(card);
                self.refresh_auras()?;
            }
            GameEvent::MinionSummoned { entity, .. } => {
                let entity = self
                    .state
                    .entities
                    .get_mut(&entity)
                    .ok_or(GameError::InvalidReservedEntity(pending.id))?;
                if entity.zone != Zone::SetAside {
                    return Err(GameError::InvalidReservedEntity(pending.id));
                }
                entity.zone = Zone::Removed;
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn commit_card_play(
        &mut self,
        play: PendingEvent,
        target: Option<EntityId>,
        position: Option<usize>,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let GameEvent::CardPlayed { player, card, cost } = play.event else {
            unreachable!("card play item must contain a card_played event")
        };
        let entity = self
            .state
            .entity(card)
            .cloned()
            .ok_or(GameError::UnknownEntity(card))?;
        if entity.zone != Zone::SetAside {
            return Err(GameError::InvalidReservedEntity(play.id));
        }
        if play.cancelled {
            return self.counter_reserved_card(play.id, player, card, queue);
        }

        let definition = self
            .runtime
            .definition(&entity.card_id)
            .cloned()
            .ok_or_else(|| GameError::UnknownCard(entity.card_id.clone()))?;
        let is_secret =
            definition.secret || self.keyword_bool(card, "enters_secret_zone", false, None)?;
        let magnetic_target = self.magnetic_target(card, position)?;
        let no_space = ((matches!(definition.kind, CardKind::Minion | CardKind::Location)
            && self.state.player(player).board.len() >= MAX_BOARD_SIZE)
            && magnetic_target.is_none())
            || (is_secret && self.state.player(player).secrets.len() >= MAX_SECRET_SIZE);
        if no_space {
            return self.counter_reserved_card(play.id, player, card, queue);
        }
        if definition.kind == CardKind::Weapon {
            return self.stage_weapon_equip(play.id, cost, player, card, target, queue);
        }

        let mut items = Vec::new();
        match definition.kind {
            CardKind::Minion | CardKind::Location => {
                if let Some(target) = magnetic_target {
                    let attachment = self.state.entities[&card].clone();
                    let enchantment_id = EnchantmentId(self.state.next_enchantment_id);
                    self.state.next_enchantment_id += 1;
                    let target_state = self.state.entities.get_mut(&target).unwrap();
                    target_state.enchantments.push(Enchantment {
                        id: enchantment_id,
                        source: card,
                        attack: attachment.attack,
                        health: attachment.max_health,
                        modifiers: (attachment.spell_damage != 0)
                            .then_some(StatModifier {
                                stat: Stat::SpellDamage,
                                operation: ModifierOperation::Add,
                                value: attachment.spell_damage,
                            })
                            .into_iter()
                            .collect(),
                        keywords: attachment.keywords.clone(),
                        silenciable: true,
                        expires_at: None,
                    });
                    target_state.attached_cards.push(attachment.card_id);
                    Self::recompute_entity(target_state);
                    self.state.entities.get_mut(&card).unwrap().zone = Zone::Removed;
                } else {
                    let position = position
                        .unwrap_or(self.state.player(player).board.len())
                        .min(self.state.player(player).board.len());
                    self.move_to_board_at(card, player, position)?;
                }
            }
            CardKind::Spell => {
                if is_secret {
                    self.move_to_secret(card, player);
                } else {
                    self.move_to_graveyard(card, player);
                }
            }
            CardKind::Hero => {
                let old_hero = self.state.player(player).hero;
                let old_state = self.state.entities[&old_hero].clone();
                let old_power = self.state.player(player).hero_power;
                let power_id = definition
                    .hero_power
                    .as_deref()
                    .expect("validated hero card must declare a hero power");
                let new_power = self.instantiate(power_id, player, Zone::HeroPower)?;

                self.state.entities.get_mut(&old_hero).unwrap().zone = Zone::Removed;
                self.state.entities.get_mut(&old_power).unwrap().zone = Zone::Removed;
                let timestamp = self.state.next_timestamp;
                self.state.next_timestamp += 1;
                {
                    let hero = self.state.entities.get_mut(&card).unwrap();
                    hero.zone = Zone::Hero;
                    hero.controller = player;
                    hero.base_health = old_state.max_health;
                    hero.max_health = old_state.max_health;
                    hero.damage = old_state.damage;
                    hero.armor = old_state.armor.saturating_add(definition.armor);
                    hero.frozen = old_state.frozen;
                    hero.frozen_since_turn = old_state.frozen_since_turn;
                    hero.attacks_this_turn = old_state.attacks_this_turn;
                    hero.exhausted = false;
                    hero.timestamp = timestamp;
                }
                let state = self.state.player_mut(player);
                state.hero = card;
                state.hero_power = new_power;
                state.hero_power_used = false;
                state.hero_power_uses_this_turn = 0;

                for event in [
                    GameEvent::HeroReplaced {
                        player,
                        old: old_hero,
                        new: card,
                    },
                    GameEvent::HeroPowerReplaced {
                        source: card,
                        player,
                        old: old_power,
                        new: new_power,
                    },
                    GameEvent::ArmorGained {
                        source: card,
                        target: card,
                        amount: definition.armor,
                    },
                ] {
                    let pending = self.begin_event(event)?;
                    items.push(ResolutionItem::PublishAfter {
                        id: pending.id,
                        event: pending.event,
                    });
                }
            }
            CardKind::Weapon => unreachable!("weapons use staged equipment"),
            CardKind::HeroPower => return Err(GameError::CardNotInHand(card)),
        }
        self.refresh_auras()?;

        if definition.kind == CardKind::Spell
            && let Some(target) = target
        {
            let targeted = self.begin_event(GameEvent::SpellTargeted {
                player,
                spell: card,
                target,
                generated_by: None,
            })?;
            items.extend(
                self.publish_after(targeted.id, targeted.event.clone())?
                    .into_iter()
                    .map(ResolutionItem::Effect),
            );
            let target_was_friendly_minion = self.state.entity(target).is_some_and(|entity| {
                entity.controller == player && entity.kind == CardKind::Minion
            });
            items.push(ResolutionItem::ResolvePlayedSpell {
                target_event: targeted,
                card_play_id: play.id,
                cost,
                secret: is_secret,
                declared_target: target,
                target_was_friendly_minion,
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
        items.push(ResolutionItem::PublishAfter {
            id: play.id,
            event: GameEvent::CardPlayed { player, card, cost },
        });
        if definition.kind == CardKind::Minion {
            let played_event = self.begin_event(GameEvent::MinionPlayed {
                player,
                minion: card,
            })?;
            items.push(ResolutionItem::PublishAfter {
                id: played_event.id,
                event: played_event.event,
            });
        }
        if definition.kind == CardKind::Location {
            let played_event = self.begin_event(GameEvent::LocationPlayed {
                player,
                location: card,
            })?;
            items.push(ResolutionItem::PublishAfter {
                id: played_event.id,
                event: played_event.event,
            });
        }
        let secondary = match definition.kind {
            CardKind::Minion if magnetic_target.is_some() => Some(GameEvent::Magnetized {
                player,
                attachment: card,
                target: magnetic_target.unwrap(),
            }),
            CardKind::Minion => Some(GameEvent::MinionSummoned {
                player,
                entity: card,
            }),
            CardKind::Weapon => unreachable!("weapons use staged equipment"),
            CardKind::Location => None,
            CardKind::Spell if is_secret => Some(GameEvent::SecretPlayed {
                player,
                secret: card,
            }),
            _ => None,
        };
        if let Some(event) = secondary {
            let event = self.begin_event(event)?;
            items.push(ResolutionItem::PublishAfter {
                id: event.id,
                event: event.event,
            });
        }
        if definition.kind == CardKind::Spell {
            let target_was_friendly_minion = target.is_some_and(|target| {
                self.state.entity(target).is_some_and(|entity| {
                    entity.controller == player && entity.kind == CardKind::Minion
                })
            });
            let cast = self.begin_event(GameEvent::SpellCast {
                player,
                spell: card,
                generated_by: None,
                target,
                cost,
                target_was_friendly_minion,
            })?;
            items.push(ResolutionItem::PublishAfter {
                id: cast.id,
                event: cast.event,
            });
        }
        for item in items.into_iter().rev() {
            queue.push_front(item);
        }
        Ok(())
    }

    pub(super) fn resolve_played_spell(
        &mut self,
        target_event: PendingEvent,
        card_play_id: EventId,
        cost: u8,
        secret: bool,
        declared_target: EntityId,
        target_was_friendly_minion: bool,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let GameEvent::SpellTargeted {
            player,
            spell,
            target,
            generated_by: None,
        } = target_event.event
        else {
            return Err(GameError::EventSpellTargetNotReplaceable(target_event.id));
        };
        let mut items = self
            .runtime
            .on_play(&self.state, spell, Some(target))
            .map_err(GameError::Script)?
            .into_iter()
            .map(ResolutionItem::Effect)
            .collect::<Vec<_>>();
        items.push(ResolutionItem::PublishAfter {
            id: card_play_id,
            event: GameEvent::CardPlayed {
                player,
                card: spell,
                cost,
            },
        });
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
            generated_by: None,
            target: Some(declared_target),
            cost,
            target_was_friendly_minion,
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

    pub(super) fn commit_hero_power(
        &mut self,
        use_event: PendingEvent,
        target: Option<EntityId>,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let GameEvent::HeroPowerUsed {
            player,
            hero_power,
            target: declared_target,
        } = use_event.event
        else {
            unreachable!("hero power item must contain a hero_power_used event")
        };
        debug_assert_eq!(declared_target, target);
        if use_event.cancelled {
            return Ok(());
        }
        let mut items = self
            .runtime
            .on_play(&self.state, hero_power, target)
            .map_err(GameError::Script)?
            .into_iter()
            .map(ResolutionItem::Effect)
            .collect::<Vec<_>>();
        items.push(ResolutionItem::PublishAfter {
            id: use_event.id,
            event: GameEvent::HeroPowerUsed {
                player,
                hero_power,
                target,
            },
        });
        for item in items.into_iter().rev() {
            queue.push_front(item);
        }
        Ok(())
    }

    pub(super) fn commit_location_use(
        &mut self,
        use_event: PendingEvent,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let GameEvent::LocationUsed {
            player,
            location,
            target,
        } = use_event.event
        else {
            unreachable!("location use item must contain location_used")
        };
        let spent = self
            .state
            .entity(location)
            .is_some_and(|entity| entity.kind == CardKind::Location && entity.health() <= 0);
        let still_present = self.state.entity(location).is_some_and(|entity| {
            entity.kind == CardKind::Location
                && entity.zone == Zone::Board
                && entity.controller == player
        });

        let mut items = Vec::new();
        if spent {
            items.push(ResolutionItem::DestroySpentLocation { player, location });
        }
        if !use_event.cancelled && still_present {
            items.extend(
                self.runtime
                    .on_location_use(&self.state, location, target)
                    .map_err(GameError::Script)?
                    .into_iter()
                    .map(ResolutionItem::Effect),
            );
            items.push(ResolutionItem::PublishAfter {
                id: use_event.id,
                event: GameEvent::LocationUsed {
                    player,
                    location,
                    target,
                },
            });
        }
        for item in items.into_iter().rev() {
            queue.push_front(item);
        }
        Ok(())
    }

    pub(super) fn destroy_spent_location(
        &mut self,
        player: PlayerId,
        location: EntityId,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let still_present = self.state.entity(location).is_some_and(|entity| {
            entity.kind == CardKind::Location
                && entity.zone == Zone::Board
                && entity.controller == player
        });
        if !still_present {
            return Ok(());
        }
        self.remove_from_zone(location, Zone::Board, player);
        self.move_to_graveyard(location, player);
        self.refresh_auras()?;
        let destroyed = self.begin_event(GameEvent::LocationDestroyed { player, location })?;
        queue.push_front(ResolutionItem::PublishAfter {
            id: destroyed.id,
            event: destroyed.event,
        });
        Ok(())
    }

    pub(super) fn stage_weapon_equip(
        &mut self,
        card_play_id: EventId,
        card_cost: u8,
        player: PlayerId,
        weapon: EntityId,
        target: Option<EntityId>,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let equip = self.begin_event(GameEvent::WeaponEquipped { player, weapon })?;
        let before = self.trigger_event(&equip, EventTiming::Before)?;
        queue.push_front(ResolutionItem::CommitWeaponEquip {
            equip,
            card_play_id,
            card_cost,
            target,
            replacement: None,
        });
        Self::prepend_effects(queue, before);
        Ok(())
    }

    pub(super) fn commit_weapon_equip(
        &mut self,
        equip: PendingEvent,
        card_play_id: EventId,
        card_cost: u8,
        target: Option<EntityId>,
        replacement: Option<PendingEvent>,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let GameEvent::WeaponEquipped { player, weapon } = equip.event else {
            unreachable!("weapon equip item must contain a weapon_equipped event")
        };
        if self
            .state
            .entity(weapon)
            .is_none_or(|entity| entity.zone != Zone::SetAside)
        {
            return Err(GameError::InvalidReservedEntity(equip.id));
        }

        if equip.cancelled {
            return self.finish_weapon_play(
                card_play_id,
                card_cost,
                equip.id,
                player,
                weapon,
                target,
                None,
                false,
                queue,
            );
        }

        if replacement.is_none()
            && let Some(old_weapon) = self.state.player(player).weapon
        {
            let destruction = self.begin_event(GameEvent::WeaponDestroyed {
                player,
                weapon: old_weapon,
            })?;
            let before = self.trigger_event(&destruction, EventTiming::Before)?;
            queue.push_front(ResolutionItem::CommitWeaponEquip {
                equip,
                card_play_id,
                card_cost,
                target,
                replacement: Some(destruction),
            });
            Self::prepend_effects(queue, before);
            return Ok(());
        }

        let mut destroyed = None;
        if let Some(replacement) = replacement {
            let GameEvent::WeaponDestroyed {
                player: replacement_player,
                weapon: old_weapon,
            } = replacement.event
            else {
                unreachable!("replacement item must contain weapon_destroyed")
            };
            debug_assert_eq!(replacement_player, player);
            if replacement.cancelled {
                return self.finish_weapon_play(
                    card_play_id,
                    card_cost,
                    equip.id,
                    player,
                    weapon,
                    target,
                    None,
                    false,
                    queue,
                );
            }
            if self.state.player(player).weapon == Some(old_weapon) {
                self.destroy_weapon(player, old_weapon);
                destroyed = Some((replacement.id, old_weapon));
            }
        }

        self.finish_weapon_play(
            card_play_id,
            card_cost,
            equip.id,
            player,
            weapon,
            target,
            destroyed,
            true,
            queue,
        )
    }

    pub(super) fn commit_effect_weapon_equip(
        &mut self,
        equip: PendingEvent,
        replacement: Option<PendingEvent>,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let GameEvent::WeaponEquipped { player, weapon } = equip.event else {
            unreachable!("effect weapon equip item must contain weapon_equipped")
        };
        if self
            .state
            .entity(weapon)
            .is_none_or(|entity| entity.zone != Zone::SetAside)
        {
            return Err(GameError::InvalidReservedEntity(equip.id));
        }

        if equip.cancelled {
            self.state.entities.get_mut(&weapon).unwrap().zone = Zone::Removed;
            self.refresh_auras()?;
            return Ok(());
        }

        if replacement.is_none()
            && let Some(old_weapon) = self.state.player(player).weapon
        {
            let destruction = self.begin_event(GameEvent::WeaponDestroyed {
                player,
                weapon: old_weapon,
            })?;
            let before = self.trigger_event(&destruction, EventTiming::Before)?;
            queue.push_front(ResolutionItem::CommitEffectWeaponEquip {
                equip,
                replacement: Some(destruction),
            });
            Self::prepend_effects(queue, before);
            return Ok(());
        }

        let mut destroyed = None;
        if let Some(replacement) = replacement {
            let GameEvent::WeaponDestroyed {
                player: replacement_player,
                weapon: old_weapon,
            } = replacement.event
            else {
                unreachable!("replacement item must contain weapon_destroyed")
            };
            debug_assert_eq!(replacement_player, player);
            if replacement.cancelled {
                self.state.entities.get_mut(&weapon).unwrap().zone = Zone::Removed;
                self.refresh_auras()?;
                return Ok(());
            }
            if self.state.player(player).weapon == Some(old_weapon) {
                self.destroy_weapon(player, old_weapon);
                destroyed = Some((replacement.id, old_weapon));
            }
        }

        self.equip_weapon_into_empty_slot(weapon, player);
        self.refresh_auras()?;
        queue.push_front(ResolutionItem::PublishAfter {
            id: equip.id,
            event: GameEvent::WeaponEquipped { player, weapon },
        });
        if let Some((id, old_weapon)) = destroyed {
            queue.push_front(ResolutionItem::PublishAfter {
                id,
                event: GameEvent::WeaponDestroyed {
                    player,
                    weapon: old_weapon,
                },
            });
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn finish_weapon_play(
        &mut self,
        card_play_id: EventId,
        card_cost: u8,
        equip_id: EventId,
        player: PlayerId,
        weapon: EntityId,
        target: Option<EntityId>,
        destroyed: Option<(EventId, EntityId)>,
        equip_succeeds: bool,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        if equip_succeeds {
            self.equip_weapon_into_empty_slot(weapon, player);
        } else {
            self.move_to_graveyard(weapon, player);
        }
        self.refresh_auras()?;

        let mut items = self
            .runtime
            .on_play(&self.state, weapon, target)
            .map_err(GameError::Script)?
            .into_iter()
            .map(ResolutionItem::Effect)
            .collect::<Vec<_>>();
        items.push(ResolutionItem::PublishAfter {
            id: card_play_id,
            event: GameEvent::CardPlayed {
                player,
                card: weapon,
                cost: card_cost,
            },
        });
        let played = self.begin_event(GameEvent::WeaponPlayed { player, weapon })?;
        items.push(ResolutionItem::PublishAfter {
            id: played.id,
            event: played.event,
        });
        if let Some((id, old_weapon)) = destroyed {
            items.push(ResolutionItem::PublishAfter {
                id,
                event: GameEvent::WeaponDestroyed {
                    player,
                    weapon: old_weapon,
                },
            });
        }
        if equip_succeeds {
            items.push(ResolutionItem::PublishAfter {
                id: equip_id,
                event: GameEvent::WeaponEquipped { player, weapon },
            });
        }
        for item in items.into_iter().rev() {
            queue.push_front(item);
        }
        Ok(())
    }

    pub(super) fn counter_reserved_card(
        &mut self,
        id: EventId,
        player: PlayerId,
        card: EntityId,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let entity = self
            .state
            .entities
            .get_mut(&card)
            .ok_or(GameError::UnknownEntity(card))?;
        if entity.zone != Zone::SetAside {
            return Err(GameError::InvalidReservedEntity(id));
        }
        entity.zone = Zone::Graveyard;
        self.state.player_mut(player).graveyard.push(card);
        self.refresh_auras()?;
        queue.push_front(ResolutionItem::PublishAfter {
            id,
            event: GameEvent::CardCountered { player, card },
        });
        Ok(())
    }

    pub(super) fn commit_discard(
        &mut self,
        discard: PendingEvent,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let GameEvent::CardDiscarded {
            source,
            player,
            card,
        } = discard.event
        else {
            unreachable!("discard item must contain card_discarded")
        };
        if discard.cancelled || !self.state.player(player).hand.contains(&card) {
            return Ok(());
        }
        let Some(entity) = self.state.entity(card) else {
            return Err(GameError::UnknownEntity(card));
        };
        if entity.zone != Zone::Hand || entity.controller != player {
            return Ok(());
        }
        let discarded_card_id = entity.card_id.clone();

        self.remove_from_zone(card, Zone::Hand, player);
        self.move_to_graveyard(card, player);
        self.state
            .player_mut(player)
            .discarded_cards_history
            .push(card);
        self.state
            .player_mut(player)
            .discarded_card_ids_history
            .push(discarded_card_id);
        self.refresh_auras()?;
        let zone_change = self.begin_event(GameEvent::ZoneChanged {
            entity: card,
            from: Zone::Hand,
            to: Zone::Graveyard,
        })?;
        queue.push_front(ResolutionItem::PublishAfterGroup {
            events: vec![
                (
                    discard.id,
                    GameEvent::CardDiscarded {
                        source,
                        player,
                        card,
                    },
                ),
                (zone_change.id, zone_change.event),
            ],
        });
        Ok(())
    }

    pub(super) fn complete_trade(
        &mut self,
        player: PlayerId,
        card: EntityId,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let entity = self
            .state
            .entity(card)
            .ok_or(GameError::UnknownEntity(card))?;
        if entity.zone != Zone::SetAside || entity.controller != player {
            return Err(GameError::InvalidTradedCard(card));
        }

        let position = self
            .rng
            .random_range(0..=self.state.player(player).deck.len());
        self.state.random_counter = self.state.random_counter.saturating_add(1);
        self.state.entities.get_mut(&card).unwrap().zone = Zone::Deck;
        self.state.player_mut(player).deck.insert(position, card);
        self.refresh_auras()?;
        let event = self.begin_event(GameEvent::CardTraded { player, card })?;
        queue.push_front(ResolutionItem::PublishAfter {
            id: event.id,
            event: event.event,
        });
        Ok(())
    }

    pub(super) fn commit_trade_draw(
        &mut self,
        trade_draw: PendingEvent,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let GameEvent::TradeDraw {
            player,
            card,
            replacement,
        } = trade_draw.event
        else {
            unreachable!("trade draw item must contain a trade_draw event")
        };
        if trade_draw.cancelled {
            return Ok(());
        }

        queue.push_front(ResolutionItem::PublishAfter {
            id: trade_draw.id,
            event: GameEvent::TradeDraw {
                player,
                card,
                replacement,
            },
        });
        if let Some(replacement) = replacement {
            self.stage_specific_draw(player, replacement, Some(card), queue)
        } else {
            self.stage_draw(player, Some(card), queue)
        }
    }

    pub(super) fn stage_specific_draw(
        &mut self,
        player: PlayerId,
        requested: EntityId,
        source: Option<EntityId>,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let Some(position) = self
            .state
            .player(player)
            .deck
            .iter()
            .position(|entity| *entity == requested)
        else {
            return self.stage_draw(player, source, queue);
        };
        let card = self.state.player_mut(player).deck.remove(position).unwrap();
        self.stage_reserved_draw(player, card, source, queue)
    }

    pub(super) fn stage_draw(
        &mut self,
        player: PlayerId,
        source: Option<EntityId>,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let Some(card) = self.state.player_mut(player).deck.pop_front() else {
            let player_state = self.state.player_mut(player);
            player_state.fatigue += 1;
            let amount = player_state.fatigue;
            let pending = self.begin_event(GameEvent::Fatigue { player, amount })?;
            let before = self.trigger_event(&pending, EventTiming::Before)?;
            queue.push_front(ResolutionItem::CommitFatigue(pending));
            Self::prepend_effects(queue, before);
            return Ok(());
        };
        self.stage_reserved_draw(player, card, source, queue)
    }

    pub(super) fn stage_reserved_draw(
        &mut self,
        player: PlayerId,
        card: EntityId,
        source: Option<EntityId>,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        self.state.entities.get_mut(&card).unwrap().zone = Zone::SetAside;
        self.refresh_auras()?;
        let event = if self.state.player(player).hand.len() < MAX_HAND_SIZE {
            GameEvent::CardDrawn {
                player,
                card,
                source,
            }
        } else {
            GameEvent::CardBurned {
                player,
                card,
                source,
            }
        };
        let pending = self.begin_event(event)?;
        let before = self.trigger_event(&pending, EventTiming::Before)?;
        queue.push_front(ResolutionItem::CommitEvent(pending));
        Self::prepend_effects(queue, before);
        Ok(())
    }

    pub(super) fn commit_fatigue(
        &mut self,
        pending: PendingEvent,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        if pending.cancelled {
            return Ok(());
        }
        let GameEvent::Fatigue { player, amount } = pending.event else {
            unreachable!("fatigue item must contain a fatigue event")
        };
        let hero = self.state.player(player).hero;
        queue.push_front(ResolutionItem::Effect(EffectSpec::Damage {
            source: hero,
            hits: vec![(hero, i32::try_from(amount).unwrap_or(i32::MAX))],
            apply_spell_damage: false,
        }));
        queue.push_front(ResolutionItem::PublishAfter {
            id: pending.id,
            event: GameEvent::Fatigue { player, amount },
        });
        Ok(())
    }

    pub(super) fn commit_reserved_draw(
        &mut self,
        id: EventId,
        player: PlayerId,
        card: EntityId,
        source: Option<EntityId>,
        burn: bool,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let entity = self
            .state
            .entities
            .get(&card)
            .ok_or(GameError::UnknownEntity(card))?;
        if entity.zone != Zone::SetAside {
            return Err(GameError::InvalidReservedEntity(id));
        }
        let burn = burn || self.state.player(player).hand.len() >= MAX_HAND_SIZE;
        let event = if burn {
            self.state.entities.get_mut(&card).unwrap().zone = Zone::Graveyard;
            self.state.player_mut(player).graveyard.push(card);
            GameEvent::CardBurned {
                player,
                card,
                source,
            }
        } else {
            self.state.entities.get_mut(&card).unwrap().zone = Zone::Hand;
            self.state
                .entities
                .get_mut(&card)
                .unwrap()
                .entered_hand_turn = Some(self.state.turn);
            self.state.player_mut(player).hand.push(card);
            GameEvent::CardDrawn {
                player,
                card,
                source,
            }
        };
        self.refresh_auras()?;
        queue.push_front(ResolutionItem::PublishAfter { id, event });
        Ok(())
    }

    pub(super) fn commit_reserved_summon(
        &mut self,
        id: EventId,
        player: PlayerId,
        entity: EntityId,
        position: Option<usize>,
        origin: ReservedSummonOrigin,
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        let reserved = self
            .state
            .entities
            .get(&entity)
            .ok_or(GameError::UnknownEntity(entity))?;
        if reserved.zone != Zone::SetAside {
            return Err(GameError::InvalidReservedEntity(id));
        }
        if self.state.player(player).board.len() >= MAX_BOARD_SIZE {
            match &origin {
                ReservedSummonOrigin::Generated => {
                    self.state.entities.get_mut(&entity).unwrap().zone = Zone::Removed;
                }
                ReservedSummonOrigin::Deck { .. } => {
                    self.restore_reserved_recruit(id, entity, &origin)?;
                }
                ReservedSummonOrigin::Graveyard { .. } => {
                    self.restore_reserved_recruit(id, entity, &origin)?;
                }
                ReservedSummonOrigin::Removed { .. } => {
                    self.restore_reserved_recruit(id, entity, &origin)?;
                }
            }
            self.refresh_auras()?;
            return Ok(());
        }
        let position = position
            .unwrap_or(self.state.player(player).board.len())
            .min(self.state.player(player).board.len());
        self.move_to_board_at(entity, player, position)?;
        self.refresh_auras()?;
        queue.push_front(ResolutionItem::PublishAfter {
            id,
            event: GameEvent::MinionSummoned { player, entity },
        });
        Ok(())
    }

    pub(super) fn restore_reserved_recruit(
        &mut self,
        event: EventId,
        entity: EntityId,
        origin: &ReservedSummonOrigin,
    ) -> Result<(), GameError> {
        let reserved = self
            .state
            .entity(entity)
            .ok_or(GameError::UnknownEntity(entity))?;
        if reserved.zone != Zone::SetAside {
            return Err(GameError::InvalidReservedEntity(event));
        }

        match origin {
            ReservedSummonOrigin::Deck {
                player,
                position,
                previous,
                next,
            } => {
                let deck = &self.state.player(*player).deck;
                let insertion = next
                    .and_then(|anchor| deck.iter().position(|candidate| *candidate == anchor))
                    .or_else(|| {
                        previous
                            .and_then(|anchor| {
                                deck.iter().position(|candidate| *candidate == anchor)
                            })
                            .map(|index| index + 1)
                    })
                    .unwrap_or((*position).min(deck.len()));
                let restored = self.state.entities.get_mut(&entity).unwrap();
                restored.zone = Zone::Deck;
                restored.controller = *player;
                self.state
                    .player_mut(*player)
                    .deck
                    .insert(insertion, entity);
            }
            ReservedSummonOrigin::Graveyard { player, position } => {
                let restored = self.state.entities.get_mut(&entity).unwrap();
                restored.zone = Zone::Graveyard;
                restored.controller = *player;
                let position = (*position).min(self.state.player(*player).graveyard.len());
                self.state
                    .player_mut(*player)
                    .graveyard
                    .insert(position, entity);
            }
            ReservedSummonOrigin::Removed { player } => {
                let restored = self.state.entities.get_mut(&entity).unwrap();
                restored.zone = Zone::Removed;
                restored.controller = *player;
            }
            ReservedSummonOrigin::Generated => {}
        }
        Ok(())
    }

    pub(super) fn stage_fresh_copy(
        &mut self,
        player: PlayerId,
        card_id: &str,
        position: usize,
        attack: Option<i32>,
        health: i32,
        final_stats: bool,
        without_keywords: &[String],
        queue: &mut VecDeque<ResolutionItem>,
    ) -> Result<(), GameError> {
        if self.state.player(player).board.len() >= MAX_BOARD_SIZE {
            return Ok(());
        }
        let entity = self.instantiate(card_id, player, Zone::SetAside)?;
        let copy = self.state.entities.get_mut(&entity).unwrap();
        for keyword in without_keywords {
            if !copy.disabled_keywords.contains(keyword) {
                copy.disabled_keywords.push(keyword.clone());
            }
        }
        if final_stats {
            let id = EnchantmentId(self.state.next_enchantment_id);
            self.state.next_enchantment_id += 1;
            let mut modifiers = vec![StatModifier {
                stat: Stat::Health,
                operation: ModifierOperation::FinalSet,
                value: health.max(1),
            }];
            if let Some(attack) = attack {
                modifiers.push(StatModifier {
                    stat: Stat::Attack,
                    operation: ModifierOperation::FinalSet,
                    value: attack,
                });
            }
            copy.enchantments.push(Enchantment {
                id,
                source: entity,
                attack: 0,
                health: 0,
                modifiers,
                keywords: Vec::new(),
                silenciable: true,
                expires_at: None,
            });
            Self::recompute_entity(copy);
            copy.damage = 0;
        } else {
            Self::recompute_entity(copy);
            copy.damage = (copy.max_health - health.max(1)).max(0);
        }

        let pending = self.begin_event(GameEvent::MinionSummoned { player, entity })?;
        let before = self.trigger_event(&pending, EventTiming::Before)?;
        queue.push_front(ResolutionItem::CommitSummon {
            summon: pending,
            position: Some(position.min(self.state.player(player).board.len())),
            origin: ReservedSummonOrigin::Generated,
        });
        Self::prepend_effects(queue, before);
        Ok(())
    }
}
