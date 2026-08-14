use super::*;

impl<R: CardRuntime> Game<R> {
    pub(super) fn install_deck(
        &mut self,
        player: PlayerId,
        cards: Vec<String>,
    ) -> Result<(), GameError> {
        self.state.player_mut(player).starting_deck = cards.clone();
        let mut entities = Vec::new();
        for card in cards {
            let definition = self
                .runtime
                .definition(&card)
                .ok_or_else(|| GameError::UnknownCard(card.clone()))?;
            if !definition.collectible
                || !matches!(
                    definition.kind,
                    CardKind::Hero
                        | CardKind::Minion
                        | CardKind::Spell
                        | CardKind::Weapon
                        | CardKind::Location
                )
            {
                return Err(GameError::InvalidDeckCard { player, card });
            }
            let entity = self.instantiate(&card, player, Zone::Deck)?;
            self.state
                .entities
                .get_mut(&entity)
                .unwrap()
                .started_in_deck = true;
            entities.push(entity);
        }
        entities.shuffle(&mut self.rng);
        self.state.player_mut(player).deck = entities.into();
        Ok(())
    }

    pub(super) fn draw_starting_hand(
        &mut self,
        player: PlayerId,
        count: u8,
    ) -> Result<(), GameError> {
        let deck = self
            .state
            .player(player)
            .deck
            .iter()
            .copied()
            .collect::<Vec<_>>();
        let mut guaranteed = Vec::new();
        for entity in deck {
            if self.keyword_bool(entity, "starts_in_opening_hand", false, None)? {
                guaranteed.push(entity);
            }
        }
        guaranteed.truncate(usize::from(count));
        for card in &guaranteed {
            self.remove_from_zone(*card, Zone::Deck, player);
            self.state.entities.get_mut(card).unwrap().zone = Zone::Hand;
            self.state.entities.get_mut(card).unwrap().entered_hand_turn = Some(self.state.turn);
            self.state.player_mut(player).hand.push(*card);
            let effects = self.publish(GameEvent::CardDrawn {
                player,
                card: *card,
                source: None,
            })?;
            self.resolve_effects(effects)?;
        }
        for _ in guaranteed.len()..usize::from(count) {
            let events = self.draw_one(player)?;
            let mut effects = Vec::new();
            for event in events {
                effects.extend(self.publish(event)?);
            }
            self.resolve_effects(effects)?;
        }
        Ok(())
    }

    pub(super) fn instantiate(
        &mut self,
        card_id: &str,
        owner: PlayerId,
        zone: Zone,
    ) -> Result<EntityId, GameError> {
        let definition = self
            .runtime
            .definition(card_id)
            .cloned()
            .ok_or_else(|| GameError::UnknownCard(card_id.to_owned()))?;
        let id = EntityId(self.state.next_entity_id);
        self.state.next_entity_id += 1;
        let timestamp = self.state.next_timestamp;
        self.state.next_timestamp += 1;
        self.state.entities.insert(
            id,
            Self::from_definition(id, owner, zone, timestamp, &definition),
        );
        if zone == Zone::Hand {
            self.state.entities.get_mut(&id).unwrap().entered_hand_turn = Some(self.state.turn);
        }
        Ok(id)
    }

    pub(super) fn copy_card_state(&mut self, template: &Entity, copy_id: EntityId) {
        let enchantments = template
            .enchantments
            .iter()
            .map(|enchantment| {
                let mut enchantment = enchantment.clone();
                enchantment.id = EnchantmentId(self.state.next_enchantment_id);
                self.state.next_enchantment_id += 1;
                if enchantment.source == template.id {
                    enchantment.source = copy_id;
                }
                enchantment
            })
            .collect();
        let copy = self.state.entities.get_mut(&copy_id).unwrap();
        copy.base_attack = template.base_attack;
        copy.base_health = template.base_health;
        copy.base_cost = template.base_cost;
        copy.base_spell_damage = template.base_spell_damage;
        copy.base_keywords = template.base_keywords.clone();
        copy.damage = template.damage;
        copy.frozen = template.frozen;
        copy.disabled_keywords = template.disabled_keywords.clone();
        copy.enchantments = enchantments;
        copy.silenced = template.silenced;
        copy.script_data = template.script_data.clone();
        copy.attached_cards = template.attached_cards.clone();
        copy.attached_deathrattles = template.attached_deathrattles.clone();
        Self::recompute_entity(copy);
    }

    pub(super) fn draw_one(&mut self, player: PlayerId) -> Result<Vec<GameEvent>, GameError> {
        let Some(card) = self.state.player_mut(player).deck.pop_front() else {
            let player_state = self.state.player_mut(player);
            player_state.fatigue += 1;
            let amount = player_state.fatigue;
            let hero = player_state.hero;
            let event = self.apply_damage(hero, hero, amount as i32)?;
            return Ok(vec![GameEvent::Fatigue { player, amount }, event]);
        };

        if self.state.player(player).hand.len() >= MAX_HAND_SIZE {
            self.state.entities.get_mut(&card).unwrap().zone = Zone::Graveyard;
            self.state.player_mut(player).graveyard.push(card);
            self.refresh_auras()?;
            Ok(vec![GameEvent::CardBurned {
                player,
                card,
                source: None,
            }])
        } else {
            self.state.entities.get_mut(&card).unwrap().zone = Zone::Hand;
            self.state
                .entities
                .get_mut(&card)
                .unwrap()
                .entered_hand_turn = Some(self.state.turn);
            self.state.player_mut(player).hand.push(card);
            self.refresh_auras()?;
            Ok(vec![GameEvent::CardDrawn {
                player,
                card,
                source: None,
            }])
        }
    }

    pub(super) fn remove_from_zone(&mut self, entity: EntityId, zone: Zone, player: PlayerId) {
        match zone {
            Zone::Deck => self
                .state
                .player_mut(player)
                .deck
                .retain(|candidate| *candidate != entity),
            Zone::Hand | Zone::Board | Zone::Secret | Zone::Graveyard => {
                let player = self.state.player_mut(player);
                let collection = match zone {
                    Zone::Hand => &mut player.hand,
                    Zone::Board => &mut player.board,
                    Zone::Secret => &mut player.secrets,
                    Zone::Graveyard => &mut player.graveyard,
                    _ => unreachable!(),
                };
                if let Some(position) = collection.iter().position(|candidate| *candidate == entity)
                {
                    collection.remove(position);
                }
            }
            Zone::Weapon => {
                if self.state.player(player).weapon == Some(entity) {
                    self.state.player_mut(player).weapon = None;
                }
            }
            Zone::Hero | Zone::SetAside | Zone::HeroPower | Zone::Removed => {}
        }
    }

    pub(super) fn move_to_board_at(
        &mut self,
        entity: EntityId,
        player: PlayerId,
        position: usize,
    ) -> Result<(), GameError> {
        debug_assert!(position <= self.state.player(player).board.len());
        let timestamp = self.state.next_timestamp;
        self.state.next_timestamp += 1;
        let card = self.state.entities.get_mut(&entity).unwrap();
        card.zone = Zone::Board;
        card.controller = player;
        card.exhausted = true;
        card.attack_at_death = None;
        card.death_source = None;
        card.timestamp = timestamp;
        self.state.player_mut(player).board.insert(position, entity);
        let ready = self.keyword_bool(entity, "ready_on_summon", false, None)?;
        self.state.entities.get_mut(&entity).unwrap().exhausted = !ready;
        Ok(())
    }

    pub(super) fn equip_weapon_into_empty_slot(&mut self, entity: EntityId, player: PlayerId) {
        debug_assert!(self.state.player(player).weapon.is_none());
        let timestamp = self.state.next_timestamp;
        self.state.next_timestamp += 1;
        let weapon = self.state.entities.get_mut(&entity).unwrap();
        weapon.zone = Zone::Weapon;
        weapon.controller = player;
        weapon.exhausted = false;
        weapon.timestamp = timestamp;
        self.state.player_mut(player).weapon = Some(entity);
    }

    pub(super) fn destroy_weapon(&mut self, player: PlayerId, weapon: EntityId) {
        let card_id = self.state.entities[&weapon].card_id.clone();
        self.state
            .player_mut(player)
            .weapons_destroyed_history
            .push(card_id);
        if self.state.player(player).weapon == Some(weapon) {
            self.state.player_mut(player).weapon = None;
        }
        self.state.entities.get_mut(&weapon).unwrap().zone = Zone::Graveyard;
        if !self.state.player(player).graveyard.contains(&weapon) {
            self.state.player_mut(player).graveyard.push(weapon);
        }
    }

    pub(super) fn move_to_graveyard(&mut self, entity: EntityId, player: PlayerId) {
        self.state.entities.get_mut(&entity).unwrap().zone = Zone::Graveyard;
        self.state.player_mut(player).graveyard.push(entity);
    }

    pub(super) fn move_to_secret(&mut self, entity: EntityId, player: PlayerId) {
        let timestamp = self.state.next_timestamp;
        self.state.next_timestamp += 1;
        let secret = self.state.entities.get_mut(&entity).unwrap();
        secret.zone = Zone::Secret;
        secret.controller = player;
        secret.timestamp = timestamp;
        self.state.player_mut(player).secrets.push(entity);
    }

    pub(super) fn from_definition(
        id: EntityId,
        owner: PlayerId,
        zone: Zone,
        timestamp: u64,
        definition: &CardDefinition,
    ) -> Entity {
        Entity {
            id,
            card_id: definition.id.clone(),
            name: definition.name.clone(),
            kind: definition.kind,
            owner,
            controller: owner,
            zone,
            base_attack: definition.attack,
            base_health: definition.health,
            base_cost: definition.cost,
            base_spell_damage: 0,
            base_keywords: definition.keywords.clone(),
            attack: definition.attack,
            max_health: definition.health,
            damage: 0,
            armor: 0,
            cost: definition.cost,
            spell_damage: 0,
            exhausted: zone == Zone::Board,
            frozen: false,
            frozen_since_turn: None,
            attacks_this_turn: 0,
            location_cooldown: 0,
            timestamp,
            keywords: definition.keywords.clone(),
            disabled_keywords: Vec::new(),
            aura_attack: 0,
            aura_health: 0,
            aura_cost: 0,
            aura_cost_set: None,
            aura_spell_damage: 0,
            aura_keywords: Vec::new(),
            enchantments: Vec::new(),
            silenced: false,
            cards_played_before: 0,
            attack_at_death: None,
            death_source: None,
            temporary_control: None,
            started_in_deck: false,
            hand_position_before_play: None,
            entered_hand_turn: (zone == Zone::Hand).then_some(0),
            script_data: Default::default(),
            attached_cards: Vec::new(),
            attached_deathrattles: Vec::new(),
        }
    }

    pub(super) fn hero(id: EntityId, owner: PlayerId, timestamp: u64) -> Entity {
        Entity {
            id,
            card_id: "builtin_hero".into(),
            name: format!("{} Hero", owner),
            kind: CardKind::Hero,
            owner,
            controller: owner,
            zone: Zone::Hero,
            base_attack: 0,
            base_health: 30,
            base_cost: 0,
            base_spell_damage: 0,
            base_keywords: Vec::new(),
            attack: 0,
            max_health: 30,
            damage: 0,
            armor: 0,
            cost: 0,
            spell_damage: 0,
            exhausted: false,
            frozen: false,
            frozen_since_turn: None,
            attacks_this_turn: 0,
            location_cooldown: 0,
            timestamp,
            keywords: Vec::new(),
            disabled_keywords: Vec::new(),
            aura_attack: 0,
            aura_health: 0,
            aura_cost: 0,
            aura_cost_set: None,
            aura_spell_damage: 0,
            aura_keywords: Vec::new(),
            enchantments: Vec::new(),
            silenced: false,
            cards_played_before: 0,
            attack_at_death: None,
            death_source: None,
            temporary_control: None,
            started_in_deck: false,
            hand_position_before_play: None,
            entered_hand_turn: None,
            script_data: Default::default(),
            attached_cards: Vec::new(),
            attached_deathrattles: Vec::new(),
        }
    }

    pub(super) fn recompute_entity(entity: &mut Entity) {
        entity.attack = Self::layered_stat(entity, Stat::Attack, entity.base_attack).max(0);
        entity.max_health = Self::layered_stat(entity, Stat::Health, entity.base_health).max(1);
        entity.cost = Self::layered_stat(entity, Stat::Cost, i32::from(entity.base_cost))
            .clamp(0, i32::from(u8::MAX)) as u8;
        entity.spell_damage = if entity.silenced {
            0
        } else {
            Self::layered_stat(entity, Stat::SpellDamage, entity.base_spell_damage).max(0)
        };
        entity.keywords = if entity.silenced {
            Vec::new()
        } else {
            entity
                .base_keywords
                .iter()
                .filter(|keyword| !entity.disabled_keywords.contains(keyword))
                .cloned()
                .collect()
        };
        entity.aura_attack = 0;
        entity.aura_health = 0;
        entity.aura_cost = 0;
        entity.aura_cost_set = None;
        entity.aura_spell_damage = 0;
        entity.aura_keywords.clear();
        for enchantment in &entity.enchantments {
            for keyword in &enchantment.keywords {
                if !entity.disabled_keywords.contains(keyword) && !entity.keywords.contains(keyword)
                {
                    entity.keywords.push(keyword.clone());
                }
            }
        }
        entity.max_health = entity.max_health.max(1);
    }

    pub(super) fn keyword_i32(
        &self,
        entity: EntityId,
        rule: &str,
        initial: i32,
        other: Option<EntityId>,
    ) -> Result<i32, GameError> {
        self.runtime
            .keyword_i32_rule(&self.state, entity, rule, initial, other)
            .map_err(GameError::Script)
    }

    pub(super) fn keyword_bool(
        &self,
        entity: EntityId,
        rule: &str,
        initial: bool,
        other: Option<EntityId>,
    ) -> Result<bool, GameError> {
        self.runtime
            .keyword_bool_rule(&self.state, entity, rule, initial, other)
            .map_err(GameError::Script)
    }

    pub(super) fn max_attacks(&self, entity: EntityId) -> Result<u8, GameError> {
        Ok(self
            .keyword_i32(entity, "max_attacks", 1, None)?
            .clamp(0, i32::from(u8::MAX)) as u8)
    }

    pub(super) fn layered_stat(entity: &Entity, stat: Stat, base: i32) -> i32 {
        let mut last_final_set = None;
        for (index, enchantment) in entity.enchantments.iter().enumerate() {
            for modifier in &enchantment.modifiers {
                if modifier.stat == stat && modifier.operation == ModifierOperation::FinalSet {
                    last_final_set = Some((index, modifier.value));
                }
            }
        }
        if let Some((final_index, mut value)) = last_final_set {
            for enchantment in entity.enchantments.iter().skip(final_index + 1) {
                for modifier in &enchantment.modifiers {
                    if modifier.stat == stat && modifier.operation == ModifierOperation::Set {
                        value = modifier.value;
                    }
                }
                value = value.saturating_add(match stat {
                    Stat::Attack => enchantment.attack,
                    Stat::Health => enchantment.health,
                    Stat::Cost | Stat::SpellDamage => 0,
                });
                for modifier in &enchantment.modifiers {
                    if modifier.stat == stat && modifier.operation == ModifierOperation::Add {
                        value = value.saturating_add(modifier.value);
                    }
                }
                for modifier in &enchantment.modifiers {
                    if modifier.stat == stat && modifier.operation == ModifierOperation::Multiply {
                        value = value.saturating_mul(modifier.value);
                    }
                }
            }
            return value;
        }
        let mut value = base;
        for enchantment in &entity.enchantments {
            for modifier in &enchantment.modifiers {
                if modifier.stat == stat && modifier.operation == ModifierOperation::Set {
                    value = modifier.value;
                }
            }
        }
        for enchantment in &entity.enchantments {
            value = value.saturating_add(match stat {
                Stat::Attack => enchantment.attack,
                Stat::Health => enchantment.health,
                Stat::Cost => 0,
                Stat::SpellDamage => 0,
            });
            for modifier in &enchantment.modifiers {
                if modifier.stat == stat
                    && matches!(
                        modifier.operation,
                        ModifierOperation::Add | ModifierOperation::PreFinalAdd
                    )
                {
                    value = value.saturating_add(modifier.value);
                }
            }
        }
        for enchantment in &entity.enchantments {
            for modifier in &enchantment.modifiers {
                if modifier.stat == stat && modifier.operation == ModifierOperation::Multiply {
                    value = value.saturating_mul(modifier.value);
                }
            }
        }
        value
    }

    pub(super) fn expiry_for(&self, duration: EffectDuration) -> Option<EnchantmentExpiry> {
        match duration {
            EffectDuration::Permanent => None,
            EffectDuration::UntilEndOfTurn => Some(EnchantmentExpiry::EndOfTurn {
                turn: self.state.turn,
            }),
        }
    }

    pub(super) fn expire_end_of_turn(&mut self, turn: u32) -> Result<(), GameError> {
        let expiring: Vec<_> = self
            .state
            .entities
            .values()
            .filter_map(|entity| {
                entity
                    .temporary_control
                    .as_ref()
                    .filter(|control| control.expires_at_turn <= turn)
                    .map(|control| (entity.id, control.original_controller))
            })
            .collect();
        for (entity, original_controller) in expiring {
            let Some(current) = self.state.entity(entity) else {
                continue;
            };
            let current_zone = current.zone;
            let current_controller = current.controller;
            if current_zone != Zone::Board {
                self.state
                    .entities
                    .get_mut(&entity)
                    .unwrap()
                    .temporary_control = None;
                continue;
            }
            self.state
                .entities
                .get_mut(&entity)
                .unwrap()
                .temporary_control = None;
            if current_controller == original_controller {
                continue;
            }
            let effect = if self.state.player(original_controller).board.len() < MAX_BOARD_SIZE {
                EffectSpec::ChangeController {
                    source: entity,
                    target: entity,
                    player: original_controller,
                }
            } else {
                EffectSpec::Destroy {
                    source: entity,
                    target: entity,
                }
            };
            self.resolve_effects(vec![effect])?;
        }
        for entity in self.state.entities.values_mut() {
            entity.enchantments.retain(|enchantment| {
                !matches!(
                    enchantment.expires_at,
                    Some(EnchantmentExpiry::EndOfTurn { turn: expiry }) if expiry <= turn
                )
            });
        }
        self.refresh_auras()?;
        self.resolve_effects(Vec::new())
    }

    pub(super) fn expire_start_of_turn(&mut self, player: PlayerId, turn: u32) {
        for entity in self.state.entities.values_mut() {
            entity.enchantments.retain(|enchantment| {
                !matches!(
                    enchantment.expires_at,
                    Some(EnchantmentExpiry::StartOfTurn {
                        player: expiry_player,
                        after_turn,
                    }) if expiry_player == player && after_turn < turn
                )
            });
        }
    }

    pub(super) fn refresh_auras(&mut self) -> Result<(), GameError> {
        for entity in self.state.entities.values_mut() {
            Self::recompute_entity(entity);
        }

        let mut sources: Vec<_> = self
            .state
            .entities
            .values()
            .filter(|entity| {
                !entity.silenced && !matches!(entity.zone, Zone::SetAside | Zone::Removed)
            })
            .map(|entity| (entity.timestamp, entity.id))
            .collect();
        sources.sort_unstable();
        let mut auras = Vec::new();
        for (_, source) in sources {
            auras.extend(
                self.runtime
                    .auras(&self.state, source)
                    .map_err(GameError::Script)?,
            );
        }
        for aura in auras {
            for target in aura.targets {
                let Some(entity) = self.state.entities.get_mut(&target) else {
                    continue;
                };
                entity.aura_cost = entity.aura_cost.saturating_add(aura.cost);
                if let Some(cost) = aura.cost_set {
                    entity.aura_cost_set = Some(cost);
                }
                if entity.kind != CardKind::Location {
                    entity.aura_attack = entity.aura_attack.saturating_add(aura.attack);
                    entity.aura_health = entity.aura_health.saturating_add(aura.health);
                    entity.aura_spell_damage =
                        entity.aura_spell_damage.saturating_add(aura.spell_damage);
                    for keyword in &aura.keywords {
                        if entity.disabled_keywords.contains(keyword) {
                            continue;
                        }
                        if !entity.aura_keywords.contains(keyword) {
                            entity.aura_keywords.push(keyword.clone());
                        }
                    }
                }
            }
        }
        for entity in self.state.entities.values_mut() {
            entity.attack = entity.attack.saturating_add(entity.aura_attack).max(0);
            entity.max_health = entity.max_health.saturating_add(entity.aura_health).max(1);
            entity.cost = entity
                .aura_cost_set
                .unwrap_or(i32::from(entity.cost))
                .saturating_add(entity.aura_cost)
                .clamp(0, i32::from(u8::MAX)) as u8;
            for keyword in &entity.aura_keywords {
                if !entity.keywords.contains(keyword) {
                    entity.keywords.push(keyword.clone());
                }
            }
        }
        // Cost floors are folded through the same generic Lua keyword rule boundary as
        // attack limits and target rules. The engine deliberately does not know which
        // keyword supplied the floor (Echo copies are one consumer).
        let mut minimum_costs = Vec::with_capacity(self.state.entities.len());
        for id in self.state.entities.keys().copied() {
            minimum_costs.push((id, self.keyword_i32(id, "minimum_cost", 0, None)?));
        }
        for (id, minimum) in minimum_costs {
            let entity = self.state.entities.get_mut(&id).unwrap();
            entity.cost = entity.cost.max(minimum.clamp(0, i32::from(u8::MAX)) as u8);
        }
        let mut spell_damage = Vec::with_capacity(self.state.entities.len());
        for id in self.state.entities.keys().copied() {
            let base = self.keyword_i32(id, "base_spell_damage", 0, None)?;
            let entity = &self.state.entities[&id];
            let layered = if entity.silenced {
                0
            } else {
                Self::layered_stat(entity, Stat::SpellDamage, base).max(0)
            };
            spell_damage.push((id, layered.saturating_add(entity.aura_spell_damage).max(0)));
        }
        for (id, value) in spell_damage {
            self.state.entities.get_mut(&id).unwrap().spell_damage = value;
        }
        for player in [PlayerId::ONE, PlayerId::TWO] {
            if player != self.state.active_player {
                continue;
            }
            let Some(weapon) = self.state.player(player).weapon else {
                continue;
            };
            let weapon_attack = self.state.entities[&weapon].attack;
            let hero = self.state.player(player).hero;
            self.state.entities.get_mut(&hero).unwrap().attack += weapon_attack;
        }
        Ok(())
    }
}
