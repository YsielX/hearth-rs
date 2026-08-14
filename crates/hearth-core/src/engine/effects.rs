use super::*;

impl<R: CardRuntime> Game<R> {
    pub(super) fn apply_effect(&mut self, effect: EffectSpec) -> Result<Vec<GameEvent>, GameError> {
        match effect {
            EffectSpec::Damage { .. }
            | EffectSpec::DamageGroup { .. }
            | EffectSpec::Heal { .. }
            | EffectSpec::Draw { .. }
            | EffectSpec::Discard { .. }
            | EffectSpec::CastSpell { .. }
            | EffectSpec::CastDrawn { .. }
            | EffectSpec::Summon { .. }
            | EffectSpec::SummonFromHand { .. }
            | EffectSpec::SummonCopy { .. }
            | EffectSpec::SummonFreshCopy { .. }
            | EffectSpec::Recruit { .. }
            | EffectSpec::MoveEntity { .. }
            | EffectSpec::ChangeController { .. }
            | EffectSpec::Transform { .. }
            | EffectSpec::Continue { .. }
            | EffectSpec::CancelEvent { .. }
            | EffectSpec::SetEventAmount { .. }
            | EffectSpec::SetTradeDraw { .. }
            | EffectSpec::DiscoverEntities { .. } => {
                unreachable!("staged effects are handled by resolve_effect_item")
            }
            EffectSpec::GiveCard {
                source,
                player,
                card_id,
            } => {
                let zone = if self.state.player(player).hand.len() < MAX_HAND_SIZE {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                };
                let card = self.instantiate(&card_id, player, zone)?;
                if zone == Zone::Hand {
                    self.state
                        .entities
                        .get_mut(&card)
                        .unwrap()
                        .entered_hand_turn = Some(self.state.turn);
                    self.state.player_mut(player).hand.push(card);
                    Ok(vec![GameEvent::CardCreated {
                        source,
                        player,
                        card,
                    }])
                } else {
                    self.state.player_mut(player).graveyard.push(card);
                    Ok(vec![GameEvent::CardBurned { player, card }])
                }
            }
            EffectSpec::GiveCardAt {
                source,
                player,
                card_id,
                position,
            } => {
                let zone = if self.state.player(player).hand.len() < MAX_HAND_SIZE {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                };
                let card = self.instantiate(&card_id, player, zone)?;
                if zone == Zone::Hand {
                    self.state
                        .entities
                        .get_mut(&card)
                        .unwrap()
                        .entered_hand_turn = Some(self.state.turn);
                    let position = position.min(self.state.player(player).hand.len());
                    self.state.player_mut(player).hand.insert(position, card);
                    Ok(vec![GameEvent::CardCreated {
                        source,
                        player,
                        card,
                    }])
                } else {
                    self.state.player_mut(player).graveyard.push(card);
                    Ok(vec![GameEvent::CardBurned { player, card }])
                }
            }
            EffectSpec::GiveMergedMinion {
                source,
                player,
                template,
                first,
                second,
            } => {
                let template_definition = self
                    .runtime
                    .definition(&template)
                    .cloned()
                    .ok_or_else(|| GameError::UnknownCard(template.clone()))?;
                let first_definition = self
                    .runtime
                    .definition(&first)
                    .cloned()
                    .ok_or_else(|| GameError::UnknownCard(first.clone()))?;
                let second_definition = self
                    .runtime
                    .definition(&second)
                    .cloned()
                    .ok_or_else(|| GameError::UnknownCard(second.clone()))?;
                if template_definition.kind != CardKind::Minion {
                    return Err(GameError::CardCannotTransformInto(template));
                }
                if first_definition.kind != CardKind::Minion {
                    return Err(GameError::CardCannotTransformInto(first));
                }
                if second_definition.kind != CardKind::Minion {
                    return Err(GameError::CardCannotTransformInto(second));
                }
                let zone = if self.state.player(player).hand.len() < MAX_HAND_SIZE {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                };
                let card = self.instantiate(&template_definition.id, player, zone)?;
                {
                    let merged = self.state.entities.get_mut(&card).unwrap();
                    merged.base_attack = first_definition
                        .attack
                        .saturating_add(second_definition.attack);
                    merged.base_health = first_definition
                        .health
                        .saturating_add(second_definition.health)
                        .max(1);
                    merged.base_cost = first_definition
                        .cost
                        .saturating_add(second_definition.cost)
                        .min(10);
                    merged.base_keywords.clear();
                    for keyword in first_definition
                        .keywords
                        .into_iter()
                        .chain(second_definition.keywords)
                    {
                        if !merged.base_keywords.contains(&keyword) {
                            merged.base_keywords.push(keyword);
                        }
                    }
                    merged.attached_cards = vec![first_definition.id, second_definition.id];
                    Self::recompute_entity(merged);
                }
                if zone == Zone::Hand {
                    self.state
                        .entities
                        .get_mut(&card)
                        .unwrap()
                        .entered_hand_turn = Some(self.state.turn);
                    self.state.player_mut(player).hand.push(card);
                    Ok(vec![GameEvent::CardCreated {
                        source,
                        player,
                        card,
                    }])
                } else {
                    self.state.player_mut(player).graveyard.push(card);
                    Ok(vec![GameEvent::CardBurned { player, card }])
                }
            }
            EffectSpec::ShuffleCardIntoDeck {
                source,
                player,
                card_id,
            } => {
                let card = self.instantiate(&card_id, player, Zone::Deck)?;
                let position = self
                    .rng
                    .random_range(0..=self.state.player(player).deck.len());
                self.state.random_counter = self.state.random_counter.saturating_add(1);
                self.state.player_mut(player).deck.insert(position, card);
                Ok(vec![GameEvent::CardCreated {
                    source,
                    player,
                    card,
                }])
            }
            EffectSpec::ReplaceHeroPower {
                source,
                player,
                card_id,
            } => {
                let definition = self
                    .runtime
                    .definition(&card_id)
                    .ok_or_else(|| GameError::UnknownCard(card_id.clone()))?;
                if definition.kind != CardKind::HeroPower {
                    return Err(GameError::InvalidHeroPower(card_id));
                }
                let definition_id = definition.id.clone();
                let old = self.state.player(player).hero_power;
                self.state.entities.get_mut(&old).unwrap().zone = Zone::Removed;
                let new = self.instantiate(&definition_id, player, Zone::HeroPower)?;
                self.state.player_mut(player).hero_power = new;
                Ok(vec![GameEvent::HeroPowerReplaced {
                    source,
                    player,
                    old,
                    new,
                }])
            }
            EffectSpec::RefreshHeroPower { source: _, player } => {
                self.state.player_mut(player).hero_power_used = false;
                Ok(Vec::new())
            }
            EffectSpec::EquipWeapon {
                source: _,
                player,
                card_id,
            } => {
                let definition = self
                    .runtime
                    .definition(&card_id)
                    .ok_or_else(|| GameError::UnknownCard(card_id.clone()))?;
                if definition.kind != CardKind::Weapon {
                    return Err(GameError::CardCannotBeEquipped(card_id));
                }
                let weapon = self.instantiate(&card_id, player, Zone::SetAside)?;
                let mut events = Vec::new();
                if let Some(old_weapon) = self.state.player(player).weapon {
                    self.destroy_weapon(player, old_weapon);
                    events.push(GameEvent::WeaponDestroyed {
                        player,
                        weapon: old_weapon,
                    });
                }
                self.equip_weapon_into_empty_slot(weapon, player);
                events.push(GameEvent::WeaponEquipped { player, weapon });
                Ok(events)
            }
            EffectSpec::GainArmor {
                source,
                player,
                amount,
            } => {
                let hero = self.state.player(player).hero;
                let entity = self.state.entities.get_mut(&hero).unwrap();
                let old = entity.armor;
                entity.armor = entity.armor.saturating_add(amount.max(0));
                let gained = entity.armor - old;
                if gained == 0 {
                    Ok(Vec::new())
                } else {
                    Ok(vec![GameEvent::ArmorGained {
                        source,
                        target: hero,
                        amount: gained,
                    }])
                }
            }
            EffectSpec::Overload {
                source,
                player,
                amount,
            } => {
                if amount == 0 {
                    return Ok(Vec::new());
                }
                let player_state = self.state.player_mut(player);
                let old = player_state.overload_pending;
                player_state.overload_pending = old.saturating_add(amount);
                let queued = player_state.overload_pending - old;
                if queued == 0 {
                    Ok(Vec::new())
                } else {
                    Ok(vec![GameEvent::OverloadQueued {
                        source,
                        player,
                        amount: queued,
                    }])
                }
            }
            EffectSpec::UnlockMana {
                source,
                player,
                amount,
            } => {
                if amount == 0 {
                    return Ok(Vec::new());
                }
                let player_state = self.state.player_mut(player);
                let unlocked = amount.min(player_state.overloaded_mana);
                player_state.overloaded_mana -= unlocked;
                let permanent = player_state.mana - player_state.temporary_mana;
                let permanent_capacity = player_state.max_mana - player_state.overloaded_mana;
                player_state.mana = player_state.temporary_mana
                    + permanent.saturating_add(unlocked).min(permanent_capacity);
                if unlocked == 0 {
                    Ok(Vec::new())
                } else {
                    Ok(vec![GameEvent::ManaUnlocked {
                        source,
                        player,
                        amount: unlocked,
                    }])
                }
            }
            EffectSpec::ClearOverload { source, player } => {
                let player_state = self.state.player_mut(player);
                let pending = player_state.overload_pending;
                let locked = player_state.overloaded_mana;
                player_state.overload_pending = 0;
                player_state.overloaded_mana = 0;
                let permanent = player_state.mana - player_state.temporary_mana;
                player_state.mana = player_state.temporary_mana
                    + permanent.saturating_add(locked).min(player_state.max_mana);
                if pending == 0 && locked == 0 {
                    Ok(Vec::new())
                } else {
                    Ok(vec![GameEvent::OverloadCleared {
                        source,
                        player,
                        pending,
                        locked,
                    }])
                }
            }
            EffectSpec::GainTemporaryMana {
                source,
                player,
                amount,
            } => {
                let player_state = self.state.player_mut(player);
                let gained = amount
                    .min(u8::MAX - player_state.mana)
                    .min(u8::MAX - player_state.temporary_mana);
                player_state.mana += gained;
                player_state.temporary_mana += gained;
                if gained == 0 {
                    Ok(Vec::new())
                } else {
                    Ok(vec![GameEvent::TemporaryManaGained {
                        source,
                        player,
                        amount: gained,
                    }])
                }
            }
            EffectSpec::GainManaCrystals {
                source,
                player,
                amount,
                filled,
            } => {
                let player_state = self.state.player_mut(player);
                let gained = amount.min(10 - player_state.max_mana);
                player_state.max_mana += gained;
                if filled {
                    player_state.mana = player_state.mana.saturating_add(gained);
                }
                if gained == 0 {
                    Ok(Vec::new())
                } else {
                    Ok(vec![GameEvent::ManaCrystalsGained {
                        source,
                        player,
                        amount: gained,
                        filled,
                    }])
                }
            }
            EffectSpec::DestroyManaCrystals {
                source,
                player,
                amount,
            } => {
                let player_state = self.state.player_mut(player);
                let destroyed = amount.min(player_state.max_mana);
                player_state.max_mana -= destroyed;
                player_state.overloaded_mana =
                    player_state.overloaded_mana.min(player_state.max_mana);
                let permanent = player_state.mana - player_state.temporary_mana;
                let permanent_capacity = player_state.max_mana - player_state.overloaded_mana;
                if permanent > permanent_capacity {
                    player_state.mana -= permanent - permanent_capacity;
                }
                if destroyed == 0 {
                    Ok(Vec::new())
                } else {
                    Ok(vec![GameEvent::ManaCrystalsDestroyed {
                        source,
                        player,
                        amount: destroyed,
                    }])
                }
            }
            EffectSpec::Destroy { source: _, target } => {
                let (zone, kind, controller) = self
                    .state
                    .entity(target)
                    .map(|entity| (entity.zone, entity.kind, entity.controller))
                    .ok_or(GameError::UnknownEntity(target))?;
                if zone != Zone::Board {
                    return Ok(Vec::new());
                }
                match kind {
                    CardKind::Minion => {
                        let entity = self.state.entities.get_mut(&target).unwrap();
                        entity.damage = entity.max_health.max(1);
                        Ok(Vec::new())
                    }
                    CardKind::Location => {
                        self.remove_from_zone(target, Zone::Board, controller);
                        self.move_to_graveyard(target, controller);
                        Ok(vec![GameEvent::LocationDestroyed {
                            player: controller,
                            location: target,
                        }])
                    }
                    _ => Ok(Vec::new()),
                }
            }
            EffectSpec::Buff {
                source,
                target,
                attack,
                health,
                keywords,
                duration,
            } => {
                if self
                    .state
                    .entity(target)
                    .is_some_and(|entity| entity.kind == CardKind::Location)
                {
                    return Ok(Vec::new());
                }
                let id = EnchantmentId(self.state.next_enchantment_id);
                self.state.next_enchantment_id += 1;
                let expires_at = self.expiry_for(duration);
                let entity = self
                    .state
                    .entities
                    .get_mut(&target)
                    .ok_or(GameError::UnknownEntity(target))?;
                for keyword in &keywords {
                    entity
                        .disabled_keywords
                        .retain(|disabled| disabled != keyword);
                }
                entity.enchantments.push(Enchantment {
                    id,
                    source,
                    attack,
                    health,
                    modifiers: Vec::new(),
                    keywords,
                    silenciable: true,
                    expires_at,
                });
                Self::recompute_entity(entity);
                let on_board = entity.zone == Zone::Board;
                if on_board && self.keyword_bool(target, "ready_on_summon", false, None)? {
                    self.state.entities.get_mut(&target).unwrap().exhausted = false;
                }
                Ok(Vec::new())
            }
            EffectSpec::DisableKeyword {
                source,
                target,
                keyword,
            } => {
                let entity = self
                    .state
                    .entities
                    .get_mut(&target)
                    .ok_or(GameError::UnknownEntity(target))?;
                if !entity.disabled_keywords.contains(&keyword) {
                    entity.disabled_keywords.push(keyword.clone());
                }
                Self::recompute_entity(entity);
                Ok(vec![GameEvent::KeywordDisabled {
                    source,
                    target,
                    keyword,
                }])
            }
            EffectSpec::ModifyStat {
                source,
                target,
                modifier,
                duration,
                silenciable,
            } => {
                if self.state.entity(target).is_some_and(|entity| {
                    entity.kind == CardKind::Location && modifier.stat != Stat::Cost
                }) {
                    return Ok(Vec::new());
                }
                let id = EnchantmentId(self.state.next_enchantment_id);
                self.state.next_enchantment_id += 1;
                let expires_at = self.expiry_for(duration);
                let entity = self
                    .state
                    .entities
                    .get_mut(&target)
                    .ok_or(GameError::UnknownEntity(target))?;
                entity.enchantments.push(Enchantment {
                    id,
                    source,
                    attack: 0,
                    health: 0,
                    modifiers: vec![modifier],
                    keywords: Vec::new(),
                    silenciable,
                    expires_at,
                });
                Self::recompute_entity(entity);
                Ok(Vec::new())
            }
            EffectSpec::RemoveEnchantmentsFromSource {
                source: _,
                target,
                enchantment_source,
            } => {
                let entity = self
                    .state
                    .entities
                    .get_mut(&target)
                    .ok_or(GameError::UnknownEntity(target))?;
                entity
                    .enchantments
                    .retain(|enchantment| enchantment.source != enchantment_source);
                Self::recompute_entity(entity);
                Ok(Vec::new())
            }
            EffectSpec::SetScriptData {
                source: _,
                target,
                key,
                value,
            } => {
                if key.is_empty() || key.len() > 64 {
                    return Err(GameError::InvalidScriptDataKey);
                }
                self.state
                    .entities
                    .get_mut(&target)
                    .ok_or(GameError::UnknownEntity(target))?
                    .script_data
                    .insert(key, value);
                Ok(Vec::new())
            }
            EffectSpec::SetPlayerScriptData {
                source: _,
                player,
                key,
                value,
            } => {
                if key.is_empty() || key.len() > 64 {
                    return Err(GameError::InvalidScriptDataKey);
                }
                self.state.player_mut(player).script_data.insert(key, value);
                Ok(Vec::new())
            }
            EffectSpec::Silence { source: _, target } => {
                let entity = self
                    .state
                    .entities
                    .get_mut(&target)
                    .ok_or(GameError::UnknownEntity(target))?;
                if entity.kind == CardKind::Minion {
                    entity.silenced = true;
                    entity
                        .enchantments
                        .retain(|enchantment| !enchantment.silenciable);
                    Self::recompute_entity(entity);
                }
                Ok(Vec::new())
            }
            EffectSpec::Freeze { source, target } => {
                let entity = self
                    .state
                    .entities
                    .get_mut(&target)
                    .ok_or(GameError::UnknownEntity(target))?;
                if !matches!(entity.kind, CardKind::Minion | CardKind::Hero)
                    || !matches!(entity.zone, Zone::Board | Zone::Hero)
                {
                    return Ok(Vec::new());
                }
                entity.frozen = true;
                entity.frozen_since_turn = Some(self.state.turn);
                Ok(vec![GameEvent::Frozen { source, target }])
            }
            EffectSpec::RevealSecret { source: _, secret } => {
                let entity = self
                    .state
                    .entities
                    .get(&secret)
                    .ok_or(GameError::UnknownEntity(secret))?;
                if entity.zone != Zone::Secret {
                    return Ok(Vec::new());
                }
                let player = entity.controller;
                self.remove_from_zone(secret, Zone::Secret, player);
                self.move_to_graveyard(secret, player);
                Ok(vec![GameEvent::SecretRevealed { player, secret }])
            }
            EffectSpec::RequestChoice { .. } => unreachable!("handled by resolve_effects"),
            EffectSpec::DiscoverCards { .. } => unreachable!("handled by resolve_effects"),
            EffectSpec::RandomChoice { .. } => unreachable!("handled by resolve_effects"),
        }
    }
}
