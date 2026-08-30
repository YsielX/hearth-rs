use super::*;

impl<R: CardRuntime> Game<R> {
    pub(super) fn magnetic_target(
        &self,
        card: EntityId,
        position: Option<usize>,
    ) -> Result<Option<EntityId>, GameError> {
        let Some(position) = position else {
            return Ok(None);
        };
        let entity = self
            .state
            .entity(card)
            .ok_or(GameError::UnknownEntity(card))?;
        if entity.kind != CardKind::Minion
            || !self.keyword_bool(card, "can_magnetize", false, None)?
        {
            return Ok(None);
        }
        let Some(target) = self
            .state
            .player(entity.controller)
            .board
            .get(position)
            .copied()
        else {
            return Ok(None);
        };
        let target_entity = &self.state.entities[&target];
        if target_entity.kind != CardKind::Minion {
            return Ok(None);
        }
        let target_definition = self
            .runtime
            .definition(&target_entity.card_id)
            .ok_or_else(|| GameError::UnknownCard(target_entity.card_id.clone()))?;
        Ok(target_definition
            .tags
            .iter()
            .any(|tag| matches!(tag.as_str(), "mech" | "all"))
            .then_some(target))
    }

    pub fn valid_targets(&self, card: EntityId) -> Result<Vec<EntityId>, GameError> {
        let controller = self
            .state
            .entity(card)
            .ok_or(GameError::UnknownEntity(card))?
            .controller;
        let mut targets = self
            .runtime
            .valid_targets(&self.state, card)
            .map_err(GameError::Script)?;
        let mut filtered = Vec::with_capacity(targets.len());
        for target in targets.drain(..) {
            let Some(entity) = self.state.entity(target) else {
                continue;
            };
            let generally_targetable =
                self.keyword_bool(target, "can_be_targeted", true, Some(card))?;
            let enemy_targetable = entity.controller == controller
                || self.keyword_bool(target, "can_be_targeted_by_enemy", true, Some(card))?;
            if generally_targetable && enemy_targetable {
                filtered.push(target);
            }
        }
        Ok(filtered)
    }

    pub fn valid_location_targets(&self, location: EntityId) -> Result<Vec<EntityId>, GameError> {
        let controller = self
            .state
            .entity(location)
            .ok_or(GameError::UnknownEntity(location))?
            .controller;
        let mut targets = self
            .runtime
            .location_targets(&self.state, location)
            .map_err(GameError::Script)?;
        let mut filtered = Vec::with_capacity(targets.len());
        for target in targets.drain(..) {
            let Some(entity) = self.state.entity(target) else {
                continue;
            };
            let generally_targetable =
                self.keyword_bool(target, "can_be_targeted", true, Some(location))?;
            let enemy_targetable = entity.controller == controller
                || self.keyword_bool(target, "can_be_targeted_by_enemy", true, Some(location))?;
            if generally_targetable && enemy_targetable {
                filtered.push(target);
            }
        }
        Ok(filtered)
    }

    pub fn valid_action_targets(
        &self,
        card: EntityId,
        action: &str,
    ) -> Result<Vec<EntityId>, GameError> {
        let controller = self
            .state
            .entity(card)
            .ok_or(GameError::UnknownEntity(card))?
            .controller;
        let mut targets = self
            .runtime
            .action_targets(&self.state, card, action)
            .map_err(GameError::Script)?;
        let mut filtered = Vec::with_capacity(targets.len());
        for target in targets.drain(..) {
            let Some(entity) = self.state.entity(target) else {
                continue;
            };
            let generally_targetable =
                self.keyword_bool(target, "can_be_targeted", true, Some(card))?;
            let enemy_targetable = entity.controller == controller
                || self.keyword_bool(target, "can_be_targeted_by_enemy", true, Some(card))?;
            if generally_targetable && enemy_targetable {
                filtered.push(target);
            }
        }
        Ok(filtered)
    }

    /// Enumerates commands accepted from the active player in the current state.
    /// UIs and AIs should use this instead of duplicating rule checks.
    pub fn legal_actions(&self) -> Result<Vec<PlayerCommand>, GameError> {
        if self.state.outcome.is_some() {
            return Ok(Vec::new());
        }
        if let Some(mulligan) = &self.state.mulligan {
            let hand = &mulligan.eligible[mulligan.current_player.index()];
            let mut actions = Vec::with_capacity(1usize << hand.len());
            for mask in 0..(1usize << hand.len()) {
                let replace = hand
                    .iter()
                    .enumerate()
                    .filter_map(|(index, entity)| ((mask >> index) & 1 == 1).then_some(*entity))
                    .collect();
                actions.push(PlayerCommand::Mulligan { replace });
            }
            actions.push(PlayerCommand::Concede);
            return Ok(actions);
        }
        if let Some(pending) = &self.state.pending_input {
            let mut actions = (0..pending.options.len())
                .map(|index| PlayerCommand::Choose { index })
                .collect::<Vec<_>>();
            actions.push(PlayerCommand::Concede);
            return Ok(actions);
        }

        let player = self.state.active_player;
        let player_state = self.state.player(player);
        let mut actions = Vec::new();
        for card in player_state.hand.iter().copied() {
            let entity = &self.state.entities[&card];
            let definition = self
                .runtime
                .definition(&entity.card_id)
                .ok_or_else(|| GameError::UnknownCard(entity.card_id.clone()))?;
            let (_, available_for_card) = self.card_cost_resource(card, player)?;
            if player_state.mana >= 1
                && !player_state.deck.is_empty()
                && self.keyword_bool(card, "can_trade", false, None)?
            {
                actions.push(PlayerCommand::TradeCard { card });
            }
            let board_full = matches!(definition.kind, CardKind::Minion | CardKind::Location)
                && player_state.board.len() >= MAX_BOARD_SIZE;
            let enters_secret =
                definition.secret || self.keyword_bool(card, "enters_secret_zone", false, None)?;
            let duplicate_persistent = enters_secret
                && player_state
                    .secrets
                    .iter()
                    .any(|secret| self.state.entities[secret].card_id == entity.card_id);
            let magnetic_positions = if definition.kind == CardKind::Minion {
                (0..player_state.board.len())
                    .filter(|position| {
                        self.magnetic_target(card, Some(*position))
                            .ok()
                            .flatten()
                            .is_some()
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            if !self.keyword_bool(card, "can_play", true, None)?
                || entity.cost > available_for_card
                || (board_full && magnetic_positions.is_empty())
                || (enters_secret && player_state.secrets.len() >= MAX_SECRET_SIZE)
                || duplicate_persistent
            {
                continue;
            }
            if definition.kind == CardKind::Location {
                actions.push(PlayerCommand::PlayCard { card, target: None });
                for position in 0..player_state.board.len() {
                    actions.push(PlayerCommand::PlayCardAt {
                        card,
                        target: None,
                        position,
                    });
                }
                continue;
            }
            let targets = self.valid_targets(card)?;
            let play_positions = if board_full {
                magnetic_positions
            } else {
                (0..player_state.board.len()).collect()
            };
            let mut add_play = |target| {
                if !board_full {
                    actions.push(PlayerCommand::PlayCard { card, target });
                }
                if matches!(definition.kind, CardKind::Minion | CardKind::Location) {
                    for position in play_positions.iter().copied() {
                        actions.push(PlayerCommand::PlayCardAt {
                            card,
                            target,
                            position,
                        });
                    }
                }
            };
            if definition.target_mode.requires_target(targets.len()) {
                for target in targets {
                    add_play(Some(target));
                }
            } else {
                add_play(None);
                for target in targets {
                    add_play(Some(target));
                }
            }
        }

        let opponent = player.opponent();
        let mut all_defenders = self
            .state
            .player(opponent)
            .board
            .iter()
            .copied()
            .filter(|entity| self.state.entities[entity].kind == CardKind::Minion)
            .collect::<Vec<_>>();
        all_defenders.push(self.state.player(opponent).hero);
        let mut attackers = player_state
            .board
            .iter()
            .filter(|entity| self.state.entities[entity].kind == CardKind::Minion)
            .copied()
            .collect::<Vec<_>>();
        if self.state.entities[&player_state.hero].attack > 0 {
            attackers.push(player_state.hero);
        }
        for attacker in attackers {
            let entity = &self.state.entities[&attacker];
            if entity.frozen
                || entity.attacks_this_turn >= self.max_attacks(attacker)?
                || entity.attack <= 0
                || !self.keyword_bool(attacker, "can_attack", true, None)?
            {
                continue;
            }
            let can_attack_exhausted = |defender| {
                self.keyword_bool(
                    attacker,
                    "can_attack_while_exhausted",
                    false,
                    Some(defender),
                )
            };
            let mut defenders = Vec::new();
            for defender in all_defenders.iter().copied() {
                if self.keyword_bool(defender, "can_be_attacked", true, Some(attacker))?
                    && (!entity.exhausted || can_attack_exhausted(defender)?)
                {
                    let priority =
                        self.keyword_i32(defender, "attack_priority", 0, Some(attacker))?;
                    defenders.push((defender, priority));
                }
            }
            let priority = defenders
                .iter()
                .map(|(_, priority)| *priority)
                .max()
                .unwrap_or(0);
            actions.extend(
                defenders
                    .into_iter()
                    .filter(|(_, candidate)| *candidate == priority)
                    .map(|(defender, _)| PlayerCommand::Attack { attacker, defender }),
            );
        }

        let hero_power = player_state.hero_power;
        let max_uses = self
            .keyword_i32(hero_power, "max_uses_per_turn", 1, None)?
            .clamp(0, i32::from(u8::MAX)) as u8;
        if player_state.hero_power_uses_this_turn < max_uses {
            let entity = &self.state.entities[&hero_power];
            if entity.cost <= player_state.mana
                && !self.keyword_bool(hero_power, "hero_power_is_passive", false, None)?
            {
                let definition = self
                    .runtime
                    .definition(&entity.card_id)
                    .ok_or_else(|| GameError::UnknownCard(entity.card_id.clone()))?;
                let targets = self.valid_targets(hero_power)?;
                let requires_target = self.keyword_bool(
                    hero_power,
                    "requires_target",
                    definition.target_mode.requires_target(targets.len()),
                    None,
                )?;
                if requires_target {
                    actions.extend(
                        targets
                            .into_iter()
                            .map(|target| PlayerCommand::UseHeroPower {
                                target: Some(target),
                            }),
                    );
                } else {
                    actions.push(PlayerCommand::UseHeroPower { target: None });
                    actions.extend(
                        targets
                            .into_iter()
                            .map(|target| PlayerCommand::UseHeroPower {
                                target: Some(target),
                            }),
                    );
                }
            }
        }

        for location in player_state.board.iter().copied().filter(|entity| {
            let entity = &self.state.entities[entity];
            entity.kind == CardKind::Location
                && entity.location_cooldown == 0
                && entity.health() > 0
        }) {
            let definition = self
                .runtime
                .definition(&self.state.entities[&location].card_id)
                .ok_or_else(|| {
                    GameError::UnknownCard(self.state.entities[&location].card_id.clone())
                })?;
            let targets = self.valid_location_targets(location)?;
            if definition.target_mode.requires_target(targets.len()) {
                actions.extend(
                    targets
                        .into_iter()
                        .map(|target| PlayerCommand::UseLocation {
                            location,
                            target: Some(target),
                        }),
                );
            } else {
                actions.push(PlayerCommand::UseLocation {
                    location,
                    target: None,
                });
                actions.extend(
                    targets
                        .into_iter()
                        .map(|target| PlayerCommand::UseLocation {
                            location,
                            target: Some(target),
                        }),
                );
            }
        }

        let mut action_sources = player_state.hand.clone();
        action_sources.extend(player_state.board.iter().copied());
        for card in action_sources {
            for action in self
                .runtime
                .card_actions(&self.state, card)
                .map_err(GameError::Script)?
            {
                let required_mana = if action.spend_all_mana {
                    0
                } else {
                    action.cost
                };
                if required_mana > player_state.mana {
                    continue;
                }
                let targets = self.valid_action_targets(card, &action.id)?;
                if action.target_mode.requires_target(targets.len()) {
                    actions.extend(targets.into_iter().map(|target| {
                        PlayerCommand::UseCardAction {
                            card,
                            action: action.id.clone(),
                            target: Some(target),
                        }
                    }));
                } else {
                    actions.push(PlayerCommand::UseCardAction {
                        card,
                        action: action.id.clone(),
                        target: None,
                    });
                    actions.extend(targets.into_iter().map(|target| {
                        PlayerCommand::UseCardAction {
                            card,
                            action: action.id.clone(),
                            target: Some(target),
                        }
                    }));
                }
            }
        }

        actions.push(PlayerCommand::EndTurn);
        actions.push(PlayerCommand::Concede);
        Ok(actions)
    }

    /// Enumerates legal commands together with the Mana each command commits
    /// in the current state. Controllers should consume this projection rather
    /// than infer dynamic costs from card definitions.
    pub fn legal_action_options(&self) -> Result<Vec<LegalAction>, GameError> {
        self.legal_actions()?
            .into_iter()
            .map(|command| {
                let mana_cost = self.command_mana_cost(&command)?;
                Ok(LegalAction { command, mana_cost })
            })
            .collect()
    }

    fn command_mana_cost(&self, command: &PlayerCommand) -> Result<u8, GameError> {
        let player = self.state.input_player();
        match command {
            PlayerCommand::PlayCard { card, .. } | PlayerCommand::PlayCardAt { card, .. } => {
                let entity = self
                    .state
                    .entity(*card)
                    .ok_or(GameError::UnknownEntity(*card))?;
                let (costs_health, _) = self.card_cost_resource(*card, player)?;
                Ok(if costs_health { 0 } else { entity.cost })
            }
            PlayerCommand::TradeCard { .. } => Ok(1),
            PlayerCommand::UseHeroPower { .. } => self
                .state
                .entity(self.state.player(player).hero_power)
                .map(|entity| entity.cost)
                .ok_or(GameError::UnknownEntity(
                    self.state.player(player).hero_power,
                )),
            PlayerCommand::UseCardAction { card, action, .. } => {
                let spec = self
                    .runtime
                    .card_actions(&self.state, *card)
                    .map_err(GameError::Script)?
                    .into_iter()
                    .find(|candidate| candidate.id == *action)
                    .ok_or_else(|| GameError::CardActionUnavailable {
                        card: *card,
                        action: action.clone(),
                    })?;
                Ok(if spec.spend_all_mana {
                    self.state.player(player).mana
                } else {
                    spec.cost
                })
            }
            PlayerCommand::Mulligan { .. }
            | PlayerCommand::Attack { .. }
            | PlayerCommand::UseLocation { .. }
            | PlayerCommand::EndTurn
            | PlayerCommand::Concede
            | PlayerCommand::ConcedePlayer { .. }
            | PlayerCommand::Choose { .. } => Ok(0),
        }
    }

    /// Applies one player command transactionally. Script failures restore the previous state.
    pub fn dispatch(&mut self, command: PlayerCommand) -> Result<(), GameError> {
        if self.state.outcome.is_some() {
            return Err(GameError::GameOver);
        }
        match (&self.state.mulligan, &command) {
            (
                Some(_),
                PlayerCommand::Mulligan { .. }
                | PlayerCommand::Concede
                | PlayerCommand::ConcedePlayer { .. },
            ) => {}
            (Some(_), _) => return Err(GameError::MulliganPending),
            (None, PlayerCommand::Mulligan { .. }) => return Err(GameError::NoMulliganPending),
            (None, _) => {}
        }
        match (&self.state.pending_input, &command) {
            (
                Some(_),
                PlayerCommand::Choose { .. }
                | PlayerCommand::Concede
                | PlayerCommand::ConcedePlayer { .. },
            ) => {}
            (Some(_), _) => return Err(GameError::ChoicePending),
            (None, PlayerCommand::Choose { .. }) => return Err(GameError::NoChoicePending),
            (None, _) => {}
        }

        let recorded_command = command.clone();
        let checkpoint = self.state.clone();
        let rng_checkpoint = self.rng.clone();
        if let Err(error) = self.dispatch_inner(command) {
            self.state = checkpoint;
            self.rng = rng_checkpoint;
            return Err(error);
        }
        if let Err(message) = self.state.validate() {
            self.state = checkpoint;
            self.rng = rng_checkpoint;
            return Err(GameError::Invariant(message));
        }
        self.command_history.push(recorded_command);
        Ok(())
    }

    pub(super) fn dispatch_inner(&mut self, command: PlayerCommand) -> Result<(), GameError> {
        let command = self.randomize_command_target(command)?;
        match command {
            PlayerCommand::Mulligan { replace } => self.mulligan(replace),
            PlayerCommand::PlayCard { card, target } => self.play_card(card, target, None),
            PlayerCommand::PlayCardAt {
                card,
                target,
                position,
            } => self.play_card(card, target, Some(position)),
            PlayerCommand::TradeCard { card } => self.trade_card(card),
            PlayerCommand::UseCardAction {
                card,
                action,
                target,
            } => self.use_card_action(card, &action, target),
            PlayerCommand::Attack { attacker, defender } => self.attack(attacker, defender),
            PlayerCommand::EndTurn => self.end_turn(),
            PlayerCommand::Concede => {
                let loser = self.state.active_player;
                self.concede_player(loser)
            }
            PlayerCommand::ConcedePlayer { player } => self.concede_player(player),
            PlayerCommand::Choose { index } => self.choose(index),
            PlayerCommand::UseHeroPower { target } => self.use_hero_power(target),
            PlayerCommand::UseLocation { location, target } => self.use_location(location, target),
        }
    }

    fn concede_player(&mut self, loser: PlayerId) -> Result<(), GameError> {
        let effects = self.publish(GameEvent::Conceded { player: loser })?;
        self.resolve_effects(effects)?;
        self.finish_game(GameOutcome::Winner(loser.opponent()));
        Ok(())
    }

    fn randomize_command_target(
        &mut self,
        command: PlayerCommand,
    ) -> Result<PlayerCommand, GameError> {
        let player = self.state.active_player;
        let hero = self.state.player(player).hero;
        if !self.keyword_bool(hero, "randomize_targets", false, None)? {
            return Ok(command);
        }

        let choose = |game: &mut Self, declared: EntityId, candidates: Vec<EntityId>| -> EntityId {
            if candidates.is_empty() || !candidates.contains(&declared) {
                return declared;
            }
            let index = game.rng.random_range(0..candidates.len());
            game.state.random_counter = game.state.random_counter.saturating_add(1);
            candidates[index]
        };

        Ok(match command {
            PlayerCommand::PlayCard {
                card,
                target: Some(target),
            } => PlayerCommand::PlayCard {
                card,
                target: Some(choose(self, target, self.valid_targets(card)?)),
            },
            PlayerCommand::PlayCardAt {
                card,
                target: Some(target),
                position,
            } => PlayerCommand::PlayCardAt {
                card,
                target: Some(choose(self, target, self.valid_targets(card)?)),
                position,
            },
            PlayerCommand::UseCardAction {
                card,
                action,
                target: Some(target),
            } => PlayerCommand::UseCardAction {
                card,
                target: Some(choose(
                    self,
                    target,
                    self.valid_action_targets(card, &action)?,
                )),
                action,
            },
            PlayerCommand::Attack { attacker, defender } => {
                let candidates = self
                    .legal_actions()?
                    .into_iter()
                    .filter_map(|action| match action {
                        PlayerCommand::Attack {
                            attacker: candidate,
                            defender,
                        } if candidate == attacker => Some(defender),
                        _ => None,
                    })
                    .collect();
                PlayerCommand::Attack {
                    attacker,
                    defender: choose(self, defender, candidates),
                }
            }
            PlayerCommand::UseHeroPower {
                target: Some(target),
            } => {
                let hero_power = self.state.player(player).hero_power;
                PlayerCommand::UseHeroPower {
                    target: Some(choose(self, target, self.valid_targets(hero_power)?)),
                }
            }
            PlayerCommand::UseLocation {
                location,
                target: Some(target),
            } => PlayerCommand::UseLocation {
                location,
                target: Some(choose(self, target, self.valid_location_targets(location)?)),
            },
            command => command,
        })
    }

    pub(super) fn use_card_action(
        &mut self,
        card: EntityId,
        action_id: &str,
        target: Option<EntityId>,
    ) -> Result<(), GameError> {
        let player = self.state.active_player;
        let entity = self
            .state
            .entity(card)
            .ok_or(GameError::UnknownEntity(card))?;
        if entity.controller != player || !matches!(entity.zone, Zone::Hand | Zone::Board) {
            return Err(GameError::CardActionUnavailable {
                card,
                action: action_id.to_owned(),
            });
        }
        let action = self
            .runtime
            .card_actions(&self.state, card)
            .map_err(GameError::Script)?
            .into_iter()
            .find(|action| action.id == action_id)
            .ok_or_else(|| GameError::CardActionUnavailable {
                card,
                action: action_id.to_owned(),
            })?;
        let targets = self.valid_action_targets(card, action_id)?;
        if action.target_mode.requires_target(targets.len()) && target.is_none() {
            return Err(GameError::TargetRequired);
        }
        if target.is_some_and(|target| !targets.contains(&target)) {
            return Err(GameError::InvalidTarget(target.unwrap()));
        }
        let available = self.state.player(player).mana;
        let spent = if action.spend_all_mana {
            available
        } else {
            if action.cost > available {
                return Err(GameError::NotEnoughMana {
                    needed: action.cost,
                    available,
                });
            }
            action.cost
        };
        let temporary = self.spend_mana(player, spent);
        let mut effects = if spent == 0 {
            Vec::new()
        } else {
            self.publish(GameEvent::ManaSpent {
                player,
                source: card,
                amount: spent,
                temporary,
            })?
        };
        effects.extend(
            self.runtime
                .on_card_action(&self.state, card, action_id, spent, target)
                .map_err(GameError::Script)?,
        );
        self.resolve_effects(effects)
    }

    pub(super) fn use_location(
        &mut self,
        location: EntityId,
        target: Option<EntityId>,
    ) -> Result<(), GameError> {
        let player = self.state.active_player;
        let entity = self
            .state
            .entity(location)
            .cloned()
            .ok_or(GameError::UnknownEntity(location))?;
        if entity.kind != CardKind::Location
            || entity.zone != Zone::Board
            || entity.controller != player
            || entity.location_cooldown != 0
            || entity.health() <= 0
        {
            return Err(GameError::CannotUseLocation(location));
        }
        let definition = self
            .runtime
            .definition(&entity.card_id)
            .cloned()
            .ok_or_else(|| GameError::UnknownCard(entity.card_id.clone()))?;
        let valid_targets = self.valid_location_targets(location)?;
        if definition.target_mode.requires_target(valid_targets.len()) && target.is_none() {
            return Err(GameError::TargetRequired);
        }
        if let Some(target) = target
            && !valid_targets.contains(&target)
        {
            return Err(GameError::InvalidTarget(target));
        }

        let location_state = self.state.entities.get_mut(&location).unwrap();
        location_state.location_cooldown = 2;
        location_state.damage = location_state.damage.saturating_add(1);
        let pending = self.begin_event(GameEvent::LocationUsed {
            player,
            location,
            target,
        })?;
        let before = self.trigger_event(&pending, EventTiming::Before)?;
        let mut resolution = before
            .into_iter()
            .map(ResolutionItem::Effect)
            .collect::<Vec<_>>();
        resolution.push(ResolutionItem::CommitLocationUse(pending));
        self.resolve_items(resolution)
    }

    pub(super) fn trade_card(&mut self, card: EntityId) -> Result<(), GameError> {
        let player = self.state.active_player;
        if !self.state.player(player).hand.contains(&card) {
            return Err(GameError::CardNotInHand(card));
        }
        if !self.keyword_bool(card, "can_trade", false, None)? {
            return Err(GameError::CardNotTradeable(card));
        }
        if self.state.player(player).deck.is_empty() {
            return Err(GameError::EmptyDeck(player));
        }
        let mana = self.state.player(player).mana;
        if mana < 1 {
            return Err(GameError::NotEnoughMana {
                needed: 1,
                available: mana,
            });
        }

        let temporary = self.spend_mana(player, 1);
        self.remove_from_zone(card, Zone::Hand, player);
        self.state.entities.get_mut(&card).unwrap().zone = Zone::SetAside;
        self.refresh_auras()?;

        let mut resolution = self
            .publish(GameEvent::ManaSpent {
                player,
                source: card,
                amount: 1,
                temporary,
            })?
            .into_iter()
            .map(ResolutionItem::Effect)
            .collect::<Vec<_>>();
        let trade_draw = self.begin_event(GameEvent::TradeDraw {
            player,
            card,
            replacement: None,
        })?;
        let before = self.trigger_event(&trade_draw, EventTiming::Before)?;
        resolution.extend(before.into_iter().map(ResolutionItem::Effect));
        resolution.push(ResolutionItem::CommitTradeDraw(trade_draw));
        resolution.push(ResolutionItem::CompleteTrade { player, card });
        self.resolve_items(resolution)
    }

    pub(super) fn mulligan(&mut self, mut replace: Vec<EntityId>) -> Result<(), GameError> {
        let mulligan = self
            .state
            .mulligan
            .clone()
            .ok_or(GameError::NoMulliganPending)?;
        let player = mulligan.current_player;
        replace.sort_unstable();
        if let Some(duplicate) = replace.windows(2).find(|pair| pair[0] == pair[1]) {
            return Err(GameError::InvalidMulliganCard(duplicate[0]));
        }
        let eligible = &mulligan.eligible[player.index()];
        for card in &replace {
            if !eligible.contains(card) || !self.state.player(player).hand.contains(card) {
                return Err(GameError::InvalidMulliganCard(*card));
            }
        }

        for card in &replace {
            self.remove_from_zone(*card, Zone::Hand, player);
            self.state.entities.get_mut(card).unwrap().zone = Zone::SetAside;
        }
        for _ in 0..replace.len() {
            let Some(card) = self.state.player_mut(player).deck.pop_front() else {
                break;
            };
            self.state.entities.get_mut(&card).unwrap().zone = Zone::Hand;
            self.state
                .entities
                .get_mut(&card)
                .unwrap()
                .entered_hand_turn = Some(self.state.turn);
            self.state.player_mut(player).hand.push(card);
        }

        if !replace.is_empty() {
            let mut deck = self
                .state
                .player_mut(player)
                .deck
                .drain(..)
                .collect::<Vec<_>>();
            for card in replace {
                self.reset_after_hidden_zone_change(card, player);
                self.state.entities.get_mut(&card).unwrap().zone = Zone::Deck;
                deck.push(card);
            }
            deck.shuffle(&mut self.rng);
            self.state.random_counter = self.state.random_counter.saturating_add(1);
            self.state.player_mut(player).deck = deck.into();
        }
        self.refresh_auras()?;

        let starting_player = self.state.starting_player;
        if player == starting_player {
            let second_player = starting_player.opponent();
            self.state.active_player = second_player;
            self.state.mulligan.as_mut().unwrap().current_player = second_player;
            return Ok(());
        }

        self.state.mulligan = None;
        self.state.active_player = starting_player;
        let second_player = starting_player.opponent();
        let coin = self.instantiate(DEFAULT_COIN, second_player, Zone::Hand)?;
        self.state.player_mut(second_player).hand.push(coin);
        self.state.record_event(GameEvent::CardCreated {
            source: coin,
            player: second_player,
            card: coin,
        });
        self.start_turn(starting_player)
    }

    pub(super) fn use_hero_power(&mut self, target: Option<EntityId>) -> Result<(), GameError> {
        let player = self.state.active_player;
        let player_state = self.state.player(player);
        let max_uses = self
            .keyword_i32(player_state.hero_power, "max_uses_per_turn", 1, None)?
            .clamp(0, i32::from(u8::MAX)) as u8;
        if player_state.hero_power_uses_this_turn >= max_uses {
            return Err(GameError::HeroPowerAlreadyUsed);
        }
        let hero_power = player_state.hero_power;
        let entity = self.state.entities[&hero_power].clone();
        if self.keyword_bool(hero_power, "hero_power_is_passive", false, None)? {
            return Err(GameError::PassiveHeroPower);
        }
        if entity.cost > player_state.mana {
            return Err(GameError::NotEnoughMana {
                needed: entity.cost,
                available: player_state.mana,
            });
        }
        let definition = self
            .runtime
            .definition(&entity.card_id)
            .cloned()
            .ok_or_else(|| GameError::UnknownCard(entity.card_id.clone()))?;
        let targets = self.valid_targets(hero_power)?;
        let requires_target = self.keyword_bool(
            hero_power,
            "requires_target",
            definition.target_mode.requires_target(targets.len()),
            None,
        )?;
        if requires_target && target.is_none() {
            return Err(GameError::TargetRequired);
        }
        if let Some(target) = target
            && !targets.contains(&target)
        {
            return Err(GameError::InvalidTarget(target));
        }

        let temporary = self.spend_mana(player, entity.cost);
        let player_state = self.state.player_mut(player);
        player_state.hero_power_used = true;
        player_state.hero_power_uses = player_state.hero_power_uses.saturating_add(1);
        player_state.hero_power_uses_this_turn =
            player_state.hero_power_uses_this_turn.saturating_add(1);
        let mut resolution = if entity.cost > 0 {
            self.publish(GameEvent::ManaSpent {
                player,
                source: hero_power,
                amount: entity.cost,
                temporary,
            })?
            .into_iter()
            .map(ResolutionItem::Effect)
            .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let pending = self.begin_event(GameEvent::HeroPowerUsed {
            player,
            hero_power,
            target,
        })?;
        let before = self.trigger_event(&pending, EventTiming::Before)?;
        resolution.extend(before.into_iter().map(ResolutionItem::Effect));
        resolution.push(ResolutionItem::CommitHeroPower {
            use_event: pending,
            target,
        });
        self.resolve_items(resolution)
    }

    pub(super) fn choose(&mut self, index: usize) -> Result<(), GameError> {
        let pending = self
            .state
            .pending_input
            .take()
            .ok_or(GameError::NoChoicePending)?;
        let choice = pending
            .options
            .get(index)
            .ok_or(GameError::InvalidChoice {
                index,
                options: pending.options.len(),
            })?
            .value
            .clone();
        let mut effects = self.publish(GameEvent::ChoiceMade {
            player: pending.player,
            source: pending.source,
            index,
        })?;
        effects.extend(
            self.runtime
                .on_resume(
                    &self.state,
                    pending.source,
                    pending.continuation_owner.as_deref(),
                    &pending.resume_hook,
                    &choice,
                )
                .map_err(GameError::Script)?,
        );
        let mut resolution = effects
            .into_iter()
            .map(ResolutionItem::Effect)
            .collect::<Vec<_>>();
        resolution.extend(pending.remaining_resolution);
        self.resolve_items(resolution)
    }

    pub(super) fn spend_mana(&mut self, player: PlayerId, amount: u8) -> u8 {
        let player = self.state.player_mut(player);
        debug_assert!(amount <= player.mana);
        let temporary = amount.min(player.temporary_mana);
        player.temporary_mana -= temporary;
        player.mana -= amount;
        temporary
    }

    /// Returns whether a card pays with Health and the amount of that resource
    /// that can legally be spent. Health payments must leave the hero alive.
    fn card_cost_resource(
        &self,
        card: EntityId,
        player: PlayerId,
    ) -> Result<(bool, u8), GameError> {
        let costs_health = self.keyword_bool(card, "costs_health_instead_of_mana", false, None)?;
        let available = if costs_health {
            self.state
                .hero(player)
                .health()
                .saturating_sub(1)
                .clamp(0, i32::from(u8::MAX)) as u8
        } else {
            self.state.player(player).mana
        };
        Ok((costs_health, available))
    }

    pub(super) fn play_card(
        &mut self,
        card: EntityId,
        target: Option<EntityId>,
        position: Option<usize>,
    ) -> Result<(), GameError> {
        let player = self.state.active_player;
        if !self.state.player(player).hand.contains(&card) {
            return Err(GameError::CardNotInHand(card));
        }

        let entity = self
            .state
            .entity(card)
            .cloned()
            .ok_or(GameError::UnknownEntity(card))?;
        if !self.keyword_bool(card, "can_play", true, None)? {
            return Err(GameError::CardCannotBePlayed(card));
        }
        let definition = self
            .runtime
            .definition(&entity.card_id)
            .cloned()
            .ok_or_else(|| GameError::UnknownCard(entity.card_id.clone()))?;
        let (costs_health, available) = self.card_cost_resource(card, player)?;
        if costs_health {
            if available < entity.cost {
                return Err(GameError::NotEnoughHealth {
                    needed: entity.cost,
                    available,
                });
            }
        } else {
            if available < entity.cost {
                return Err(GameError::NotEnoughMana {
                    needed: entity.cost,
                    available,
                });
            }
        }
        if matches!(definition.kind, CardKind::Minion | CardKind::Location)
            && self.state.player(player).board.len() >= MAX_BOARD_SIZE
            && self.magnetic_target(card, position)?.is_none()
        {
            return Err(GameError::BoardFull);
        }
        if let Some(position) = position {
            let max = self.state.player(player).board.len();
            if !matches!(definition.kind, CardKind::Minion | CardKind::Location) || position > max {
                return Err(GameError::InvalidBoardPosition { position, max });
            }
        }
        let enters_secret =
            definition.secret || self.keyword_bool(card, "enters_secret_zone", false, None)?;
        if enters_secret && self.state.player(player).secrets.len() >= MAX_SECRET_SIZE {
            return Err(GameError::SecretZoneFull);
        }
        if enters_secret
            && self
                .state
                .player(player)
                .secrets
                .iter()
                .any(|secret| self.state.entities[secret].card_id == entity.card_id)
        {
            return Err(GameError::CardCannotBePlayed(card));
        }

        if definition.kind == CardKind::Location {
            if let Some(target) = target {
                return Err(GameError::InvalidTarget(target));
            }
        } else {
            let valid_targets = self.valid_targets(card)?;
            if definition.target_mode.requires_target(valid_targets.len()) && target.is_none() {
                return Err(GameError::TargetRequired);
            }
            if let Some(target) = target
                && !valid_targets.contains(&target)
            {
                return Err(GameError::InvalidTarget(target));
            }
        }

        let temporary = if costs_health {
            let hero = self.state.player(player).hero;
            self.state.entities.get_mut(&hero).unwrap().damage = self.state.entities[&hero]
                .damage
                .saturating_add(i32::from(entity.cost));
            0
        } else {
            self.spend_mana(player, entity.cost)
        };
        let hand_position = self
            .state
            .player(player)
            .hand
            .iter()
            .position(|candidate| *candidate == card);
        self.remove_from_zone(card, Zone::Hand, player);
        let cards_played_before = self.state.player(player).cards_played_this_turn;
        let played = self.state.entities.get_mut(&card).unwrap();
        played.zone = Zone::SetAside;
        played.cards_played_before = cards_played_before;
        played.hand_position_before_play = hand_position;
        let state = self.state.player_mut(player);
        state.cards_played_this_turn = state.cards_played_this_turn.saturating_add(1);
        state.cards_played_history.push(entity.card_id.clone());
        state.cards_played_current_turn.push(entity.card_id.clone());
        self.refresh_auras()?;
        let mut resolution = if entity.cost > 0 && !costs_health {
            self.publish(GameEvent::ManaSpent {
                player,
                source: card,
                amount: entity.cost,
                temporary,
            })?
            .into_iter()
            .map(ResolutionItem::Effect)
            .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let pending = self.begin_event(GameEvent::CardPlayed {
            player,
            card,
            cost: entity.cost,
        })?;
        let before = self.trigger_event(&pending, EventTiming::Before)?;
        resolution.extend(before.into_iter().map(ResolutionItem::Effect));
        resolution.push(ResolutionItem::CommitCardPlay {
            play: pending,
            target,
            position,
        });
        self.resolve_items(resolution)
    }

    pub(super) fn attack(
        &mut self,
        attacker: EntityId,
        defender: EntityId,
    ) -> Result<(), GameError> {
        let player = self.state.active_player;
        let attacker_entity = self
            .state
            .entity(attacker)
            .cloned()
            .ok_or(GameError::UnknownEntity(attacker))?;
        let defender_entity = self
            .state
            .entity(defender)
            .cloned()
            .ok_or(GameError::UnknownEntity(defender))?;

        if !matches!(attacker_entity.zone, Zone::Board | Zone::Hero)
            || !matches!(attacker_entity.kind, CardKind::Minion | CardKind::Hero)
            || attacker_entity.controller != player
            || attacker_entity.attacks_this_turn >= self.max_attacks(attacker)?
            || attacker_entity.frozen
            || attacker_entity.attack <= 0
            || !self.keyword_bool(attacker, "can_attack", true, None)?
        {
            return Err(GameError::CannotAttack(attacker));
        }
        if defender_entity.controller != player.opponent()
            || !matches!(defender_entity.zone, Zone::Board | Zone::Hero)
            || !matches!(defender_entity.kind, CardKind::Minion | CardKind::Hero)
        {
            return Err(GameError::InvalidTarget(defender));
        }
        if attacker_entity.exhausted
            && !self.keyword_bool(
                attacker,
                "can_attack_while_exhausted",
                false,
                Some(defender),
            )?
        {
            return Err(GameError::CannotAttack(attacker));
        }
        if !self.keyword_bool(defender, "can_be_attacked", true, Some(attacker))? {
            return Err(GameError::InvalidTarget(defender));
        }
        let defender_priority = self.keyword_i32(defender, "attack_priority", 0, Some(attacker))?;
        let mut max_priority = defender_priority;
        for candidate in self.state.player(player.opponent()).board.iter().copied() {
            if self.state.entities[&candidate].kind == CardKind::Minion
                && self.keyword_bool(candidate, "can_be_attacked", true, Some(attacker))?
            {
                max_priority = max_priority.max(self.keyword_i32(
                    candidate,
                    "attack_priority",
                    0,
                    Some(attacker),
                )?);
            }
        }
        if defender_priority < max_priority {
            return Err(GameError::InvalidTarget(defender));
        }

        self.state
            .entities
            .get_mut(&attacker)
            .unwrap()
            .attacks_this_turn += 1;
        let event = GameEvent::Attack {
            attacker,
            defender,
            collateral: Vec::new(),
        };
        let pending = self.begin_event(event)?;
        let before = self.trigger_event(&pending, EventTiming::Before)?;
        let mut resolution = before
            .into_iter()
            .map(ResolutionItem::Effect)
            .collect::<Vec<_>>();
        resolution.push(ResolutionItem::CommitEvent(pending));
        self.resolve_items(resolution)
    }

    pub(super) fn end_turn(&mut self) -> Result<(), GameError> {
        let player = self.state.active_player;
        let turn = self.state.turn;
        let mut characters = self.state.player(player).board.clone();
        characters.push(self.state.player(player).hero);
        for character in characters {
            let frozen_since = self.state.entities[&character].frozen_since_turn;
            if frozen_since.is_some_and(|frozen_turn| frozen_turn < turn) {
                let entity = self.state.entities.get_mut(&character).unwrap();
                entity.frozen = false;
                entity.frozen_since_turn = None;
            }
        }
        let effects = self.publish(GameEvent::TurnEnded { player, turn })?;
        self.resolve_effects(effects)?;
        self.expire_end_of_turn(turn)?;
        let expired = {
            let state = self.state.player_mut(player);
            let expired = state.temporary_mana.min(state.mana);
            state.mana -= expired;
            state.temporary_mana = 0;
            expired
        };
        if expired > 0 {
            let effects = self.publish(GameEvent::TemporaryManaExpired {
                player,
                amount: expired,
            })?;
            self.resolve_effects(effects)?;
        }
        // End-of-turn effects still get the opportunity to decide the game. If
        // neither hero has lost, the official long-game limit ends the match
        // before turn 90 can start (and therefore before its draw or triggers).
        if self.state.outcome.is_some() {
            return Ok(());
        }
        if turn >= MAX_GAME_TURNS {
            self.finish_game(GameOutcome::Draw);
            return Ok(());
        }
        let next = if self.state.player(player).extra_turns > 0 {
            self.state.player_mut(player).extra_turns -= 1;
            player
        } else {
            player.opponent()
        };
        self.start_turn(next)
    }

    pub(super) fn start_turn(&mut self, player: PlayerId) -> Result<(), GameError> {
        self.state.turn += 1;
        self.state.active_player = player;
        self.expire_start_of_turn(player, self.state.turn);
        let (board, locked) = {
            let player_state = self.state.player_mut(player);
            player_state.max_mana = (player_state.max_mana + 1).min(10);
            player_state.temporary_mana = 0;
            player_state.overloaded_mana = player_state.overload_pending.min(player_state.max_mana);
            player_state.overload_pending = 0;
            player_state.mana = player_state.max_mana - player_state.overloaded_mana;
            player_state.hero_power_used = false;
            player_state.hero_power_uses_this_turn = 0;
            player_state.cards_played_this_turn = 0;
            player_state.cards_played_last_turn =
                std::mem::take(&mut player_state.cards_played_current_turn);
            (player_state.board.clone(), player_state.overloaded_mana)
        };
        for entity in board {
            let entity = self.state.entities.get_mut(&entity).unwrap();
            match entity.kind {
                CardKind::Minion => {
                    entity.exhausted = false;
                    entity.attacks_this_turn = 0;
                }
                CardKind::Location => {
                    entity.location_cooldown = entity.location_cooldown.saturating_sub(1);
                }
                _ => {}
            }
        }
        let hero = self.state.player(player).hero;
        self.state
            .entities
            .get_mut(&hero)
            .unwrap()
            .attacks_this_turn = 0;
        self.refresh_auras()?;
        self.resolve_effects(Vec::new())?;
        if self.state.outcome.is_some() {
            return Ok(());
        }
        let mut effects = Vec::new();
        if locked > 0 {
            effects.extend(self.publish(GameEvent::ManaLocked {
                player,
                amount: locked,
            })?);
        }
        effects.extend(self.publish(GameEvent::TurnStarted {
            player,
            turn: self.state.turn,
        })?);
        self.resolve_effects(effects)?;
        self.resolve_effects(vec![EffectSpec::Draw {
            source: None,
            player,
            count: 1,
        }])
    }
}
