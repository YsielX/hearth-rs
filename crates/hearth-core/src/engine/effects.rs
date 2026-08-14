use super::*;

impl<R: CardRuntime> Game<R> {
    pub(super) fn apply_effect(&mut self, effect: EffectSpec) -> Result<Vec<GameEvent>, GameError> {
        match effect {
            EffectSpec::Damage { .. }
            | EffectSpec::DamageGroup { .. }
            | EffectSpec::DamageBatch { .. }
            | EffectSpec::Heal { .. }
            | EffectSpec::HealGroup { .. }
            | EffectSpec::Draw { .. }
            | EffectSpec::DrawEntity { .. }
            | EffectSpec::Discard { .. }
            | EffectSpec::CastSpell { .. }
            | EffectSpec::CastRandomSpells { .. }
            | EffectSpec::CastDrawn { .. }
            | EffectSpec::CastDeckSpellRandomTarget { .. }
            | EffectSpec::Summon { .. }
            | EffectSpec::SummonFromHand { .. }
            | EffectSpec::SummonExisting { .. }
            | EffectSpec::SummonCopy { .. }
            | EffectSpec::SummonFreshCopy { .. }
            | EffectSpec::Recruit { .. }
            | EffectSpec::Destroy { .. }
            | EffectSpec::DestroyGroup { .. }
            | EffectSpec::LoseWeaponDurability { .. }
            | EffectSpec::MoveEntity { .. }
            | EffectSpec::ChangeController { .. }
            | EffectSpec::ChangeControllerUntilEndOfTurn { .. }
            | EffectSpec::ForceAttack { .. }
            | EffectSpec::Transform { .. }
            | EffectSpec::TransformIntoCopy { .. }
            | EffectSpec::TransformGroup { .. }
            | EffectSpec::TransformBatch { .. }
            | EffectSpec::SwapStatsGroup { .. }
            | EffectSpec::SwapDecks { .. }
            | EffectSpec::Continue { .. }
            | EffectSpec::CancelEvent { .. }
            | EffectSpec::SetEventAmount { .. }
            | EffectSpec::SetAttackDefender { .. }
            | EffectSpec::AddAttackCollateral { .. }
            | EffectSpec::SetDamageTarget { .. }
            | EffectSpec::SetTradeDraw { .. }
            | EffectSpec::TriggerHook { .. }
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
                    Ok(vec![GameEvent::CardBurned {
                        player,
                        card,
                        source: Some(source),
                    }])
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
                    Ok(vec![GameEvent::CardBurned {
                        player,
                        card,
                        source: Some(source),
                    }])
                }
            }
            EffectSpec::GiveCopy {
                source,
                player,
                target,
                preserve_state,
                attack,
                health,
                cost,
            } => {
                let template = self
                    .state
                    .entity(target)
                    .cloned()
                    .ok_or(GameError::UnknownEntity(target))?;
                let zone = if self.state.player(player).hand.len() < MAX_HAND_SIZE {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                };
                let card = self.instantiate(&template.card_id, player, zone)?;
                if preserve_state {
                    self.copy_card_state(&template, card);
                }
                let mut modifiers = Vec::new();
                for (stat, value) in [
                    (Stat::Attack, attack),
                    (Stat::Health, health),
                    (Stat::Cost, cost),
                ] {
                    if let Some(value) = value {
                        modifiers.push(StatModifier {
                            stat,
                            operation: ModifierOperation::FinalSet,
                            value,
                        });
                    }
                }
                let modifier_id = if modifiers.is_empty() {
                    None
                } else {
                    let id = EnchantmentId(self.state.next_enchantment_id);
                    self.state.next_enchantment_id += 1;
                    Some(id)
                };
                {
                    let copy = self.state.entities.get_mut(&card).unwrap();
                    copy.damage = 0;
                    copy.frozen = false;
                    copy.frozen_since_turn = None;
                    if let Some(id) = modifier_id {
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
                    }
                    Self::recompute_entity(copy);
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
                    Ok(vec![GameEvent::CardBurned {
                        player,
                        card,
                        source: Some(source),
                    }])
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
                    Ok(vec![GameEvent::CardBurned {
                        player,
                        card,
                        source: Some(source),
                    }])
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
            EffectSpec::ShuffleCopyIntoDeck {
                source,
                player,
                target,
            } => {
                let template = self
                    .state
                    .entity(target)
                    .cloned()
                    .ok_or(GameError::UnknownEntity(target))?;
                let card = self.instantiate(&template.card_id, player, Zone::Deck)?;
                self.copy_card_state(&template, card);
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
            EffectSpec::ReplaceHero {
                source,
                player,
                card_id,
            } => {
                let definition = self
                    .runtime
                    .definition(&card_id)
                    .cloned()
                    .ok_or_else(|| GameError::UnknownCard(card_id.clone()))?;
                if definition.kind != CardKind::Hero {
                    return Err(GameError::InvalidHero(card_id));
                }
                let power_id = definition
                    .hero_power
                    .as_deref()
                    .ok_or_else(|| GameError::InvalidHero(definition.id.clone()))?;
                let power_definition = self
                    .runtime
                    .definition(power_id)
                    .ok_or_else(|| GameError::UnknownCard(power_id.to_owned()))?;
                if power_definition.kind != CardKind::HeroPower {
                    return Err(GameError::InvalidHeroPower(power_id.to_owned()));
                }

                let old = self.state.player(player).hero;
                let old_power = self.state.player(player).hero_power;
                let old_state = self.state.entities[&old].clone();
                let new = self.instantiate(&definition.id, player, Zone::Hero)?;
                let new_power = self.instantiate(power_id, player, Zone::HeroPower)?;
                self.state.entities.get_mut(&old).unwrap().zone = Zone::Removed;
                self.state.entities.get_mut(&old_power).unwrap().zone = Zone::Removed;
                {
                    let hero = self.state.entities.get_mut(&new).unwrap();
                    hero.damage = 0;
                    hero.armor = old_state.armor.saturating_add(definition.armor);
                    hero.frozen = old_state.frozen;
                    hero.frozen_since_turn = old_state.frozen_since_turn;
                    hero.attacks_this_turn = old_state.attacks_this_turn;
                    hero.exhausted = false;
                }
                let state = self.state.player_mut(player);
                state.hero = new;
                state.hero_power = new_power;
                state.hero_power_used = false;
                state.hero_power_uses_this_turn = 0;
                self.refresh_auras()?;
                Ok(vec![
                    GameEvent::HeroReplaced { player, old, new },
                    GameEvent::HeroPowerReplaced {
                        source,
                        player,
                        old: old_power,
                        new: new_power,
                    },
                ])
            }
            EffectSpec::GrantPlayerKeyword {
                source: _,
                player,
                keyword,
            } => {
                let keywords = &mut self.state.player_mut(player).keywords;
                if !keywords.contains(&keyword) {
                    keywords.push(keyword);
                    self.refresh_auras()?;
                }
                Ok(Vec::new())
            }
            EffectSpec::DisablePlayerKeyword {
                source: _,
                player,
                keyword,
            } => {
                self.state
                    .player_mut(player)
                    .keywords
                    .retain(|candidate| candidate != &keyword);
                self.refresh_auras()?;
                Ok(Vec::new())
            }
            EffectSpec::SetPlayerClass {
                source: _,
                player,
                class,
            } => {
                if class.trim().is_empty() || class.len() > 64 {
                    return Err(GameError::InvalidPlayerClass { player, class });
                }
                self.state.player_mut(player).class = class;
                Ok(Vec::new())
            }
            EffectSpec::RefreshHeroPower { source: _, player } => {
                let player = self.state.player_mut(player);
                player.hero_power_used = false;
                player.hero_power_uses_this_turn = 0;
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
            EffectSpec::LoseArmor {
                source: _,
                player,
                amount,
            } => {
                let hero = self.state.player(player).hero;
                let entity = self.state.entities.get_mut(&hero).unwrap();
                entity.armor = entity.armor.saturating_sub(amount.max(0));
                Ok(Vec::new())
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
                player_state.overload_queued_total = player_state
                    .overload_queued_total
                    .saturating_add(u32::from(queued));
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
            EffectSpec::FillManaCrystals {
                source,
                player,
                amount,
            } => {
                let target = amount.min(10);
                let player_state = self.state.player_mut(player);
                let old_max = player_state.max_mana;
                let gained = target.saturating_sub(old_max);
                let pending = player_state.overload_pending;
                let locked = player_state.overloaded_mana;
                let temporary = player_state.temporary_mana.min(player_state.mana);
                player_state.max_mana = player_state.max_mana.max(target);
                player_state.mana = player_state.max_mana;
                player_state.temporary_mana = 0;
                player_state.overload_pending = 0;
                player_state.overloaded_mana = 0;
                let mut events = Vec::new();
                if pending > 0 || locked > 0 {
                    events.push(GameEvent::OverloadCleared {
                        source,
                        player,
                        pending,
                        locked,
                    });
                }
                if temporary > 0 {
                    events.push(GameEvent::TemporaryManaExpired {
                        player,
                        amount: temporary,
                    });
                }
                if gained > 0 {
                    events.push(GameEvent::ManaCrystalsGained {
                        source,
                        player,
                        amount: gained,
                        filled: true,
                    });
                }
                Ok(events)
            }
            EffectSpec::RefreshManaCrystals { source: _, player } => {
                let player_state = self.state.player_mut(player);
                let unlocked = player_state
                    .max_mana
                    .saturating_sub(player_state.overloaded_mana);
                player_state.mana = unlocked.saturating_add(player_state.temporary_mana);
                Ok(Vec::new())
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
            EffectSpec::SpendMana {
                source,
                player,
                amount,
            } => {
                let amount = amount.min(self.state.player(player).mana);
                if amount == 0 {
                    return Ok(Vec::new());
                }
                let temporary = self.spend_mana(player, amount);
                Ok(vec![GameEvent::ManaSpent {
                    player,
                    source,
                    amount,
                    temporary,
                }])
            }
            EffectSpec::SetHealth {
                source,
                target,
                health,
            } => {
                let entity = self
                    .state
                    .entity(target)
                    .ok_or(GameError::UnknownEntity(target))?;
                if !matches!(entity.kind, CardKind::Hero | CardKind::Minion) {
                    return Ok(Vec::new());
                }
                // Auras are applied after enchantment layers. Offset the current
                // aura so the requested value is exact at resolution time.
                let value = health.max(1).saturating_sub(entity.aura_health);
                let id = EnchantmentId(self.state.next_enchantment_id);
                self.state.next_enchantment_id += 1;
                let entity = self.state.entities.get_mut(&target).unwrap();
                entity.enchantments.push(Enchantment {
                    id,
                    source,
                    attack: 0,
                    health: 0,
                    modifiers: vec![StatModifier {
                        stat: Stat::Health,
                        operation: ModifierOperation::Set,
                        value,
                    }],
                    keywords: Vec::new(),
                    silenciable: true,
                    expires_at: None,
                });
                entity.damage = 0;
                Self::recompute_entity(entity);
                Ok(Vec::new())
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
            EffectSpec::AttachDeathrattle {
                source: _,
                target,
                card_id,
            } => {
                if self.runtime.definition(&card_id).is_none() {
                    return Err(GameError::UnknownCard(card_id));
                }
                let entity = self
                    .state
                    .entities
                    .get_mut(&target)
                    .ok_or(GameError::UnknownEntity(target))?;
                if entity.kind == CardKind::Minion && entity.zone == Zone::Board {
                    entity.attached_deathrattles.push(card_id);
                }
                Ok(Vec::new())
            }
            EffectSpec::AttachScript {
                source: _,
                target,
                card_id,
            } => {
                if self.runtime.definition(&card_id).is_none() {
                    return Err(GameError::UnknownCard(card_id));
                }
                let entity = self
                    .state
                    .entities
                    .get_mut(&target)
                    .ok_or(GameError::UnknownEntity(target))?;
                if !entity.attached_cards.contains(&card_id) {
                    entity.attached_cards.push(card_id);
                }
                Ok(Vec::new())
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
            EffectSpec::ModifyStatGroup {
                source,
                targets,
                modifiers,
                duration,
                silenciable,
                reset_damage,
            } => {
                let expires_at = self.expiry_for(duration);
                let mut seen = std::collections::BTreeSet::new();
                for target in targets {
                    if !seen.insert(target) {
                        continue;
                    }
                    let Some(entity) = self.state.entities.get_mut(&target) else {
                        continue;
                    };
                    if entity.kind == CardKind::Location
                        && modifiers.iter().any(|modifier| modifier.stat != Stat::Cost)
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
                        modifiers: modifiers.clone(),
                        keywords: Vec::new(),
                        silenciable,
                        expires_at,
                    });
                    Self::recompute_entity(entity);
                    if reset_damage {
                        entity.damage = 0;
                    }
                }
                Ok(Vec::new())
            }
            EffectSpec::GrantKeywordUntilNextTurn {
                source,
                target,
                keyword,
            } => {
                let id = EnchantmentId(self.state.next_enchantment_id);
                self.state.next_enchantment_id += 1;
                let current_turn = self.state.turn;
                let entity = self
                    .state
                    .entities
                    .get_mut(&target)
                    .ok_or(GameError::UnknownEntity(target))?;
                entity
                    .disabled_keywords
                    .retain(|disabled| disabled != &keyword);
                entity.enchantments.push(Enchantment {
                    id,
                    source,
                    attack: 0,
                    health: 0,
                    modifiers: Vec::new(),
                    keywords: vec![keyword],
                    silenciable: true,
                    expires_at: Some(EnchantmentExpiry::StartOfTurn {
                        player: entity.controller,
                        after_turn: current_turn,
                    }),
                });
                Self::recompute_entity(entity);
                Ok(Vec::new())
            }
            EffectSpec::TakeExtraTurn { source: _, player } => {
                self.state.player_mut(player).extra_turns =
                    self.state.player(player).extra_turns.saturating_add(1);
                Ok(Vec::new())
            }
            EffectSpec::WinGame { source: _, player } => {
                self.finish_game(GameOutcome::Winner(player));
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
                self.refresh_auras()?;
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
                self.refresh_auras()?;
                Ok(Vec::new())
            }
            EffectSpec::IncrementPlayerScriptData {
                source,
                player,
                key,
                delta,
            } => {
                if key.is_empty() || key.len() > 64 {
                    return Err(GameError::InvalidScriptDataKey);
                }
                let old = self
                    .state
                    .player(player)
                    .script_data
                    .get(&key)
                    .copied()
                    .unwrap_or(0);
                let value = old.saturating_add(delta);
                self.state
                    .player_mut(player)
                    .script_data
                    .insert(key.clone(), value);
                self.refresh_auras()?;
                Ok(vec![GameEvent::PlayerScriptDataChanged {
                    source,
                    player,
                    key,
                    old,
                    new: value,
                }])
            }
            EffectSpec::Silence { source, target } => {
                if !self.keyword_bool(target, "can_be_silenced", true, Some(source))? {
                    return Ok(Vec::new());
                }
                let temporary_control = self
                    .state
                    .entity(target)
                    .and_then(|entity| entity.temporary_control.clone());
                let mut events = Vec::new();
                if let Some(control) = temporary_control {
                    let current = self.state.entities[&target].controller;
                    if current != control.original_controller {
                        if self.state.player(control.original_controller).board.len()
                            < MAX_BOARD_SIZE
                        {
                            self.state
                                .player_mut(current)
                                .board
                                .retain(|candidate| *candidate != target);
                            self.state
                                .player_mut(control.original_controller)
                                .board
                                .push(target);
                            self.state.entities.get_mut(&target).unwrap().controller =
                                control.original_controller;
                            events.push(GameEvent::ControllerChanged {
                                source,
                                entity: target,
                                from: current,
                                to: control.original_controller,
                            });
                        } else {
                            let entity = self.state.entities.get_mut(&target).unwrap();
                            entity.damage = entity.max_health;
                        }
                    }
                    self.state
                        .entities
                        .get_mut(&target)
                        .unwrap()
                        .temporary_control = None;
                }
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
                    entity.attached_cards.clear();
                    entity.attached_deathrattles.clear();
                    Self::recompute_entity(entity);
                }
                Ok(events)
            }
            EffectSpec::Freeze { source, target } => {
                if !self.keyword_bool(target, "can_be_frozen", true, Some(source))? {
                    return Ok(Vec::new());
                }
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
