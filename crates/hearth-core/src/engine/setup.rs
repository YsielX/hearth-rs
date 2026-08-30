use super::*;

struct GameSetupOptions {
    hero_powers: [String; 2],
    classes: [String; 2],
    enforce_deck_classes: [bool; 2],
    starting_player: PlayerId,
    sideboards: [BTreeMap<String, Vec<String>>; 2],
}

impl<R: CardRuntime> Game<R> {
    pub fn new(
        runtime: R,
        deck_one: Vec<String>,
        deck_two: Vec<String>,
        seed: u64,
    ) -> Result<Self, GameError> {
        Self::new_with_hero_powers(
            runtime,
            deck_one,
            deck_two,
            seed,
            [DEFAULT_HERO_POWER.to_owned(), DEFAULT_HERO_POWER.to_owned()],
        )
    }

    pub fn new_unrestricted(
        runtime: R,
        deck_one: Vec<String>,
        deck_two: Vec<String>,
        seed: u64,
    ) -> Result<Self, GameError> {
        Self::new_unrestricted_with_hero_powers_and_classes(
            runtime,
            deck_one,
            deck_two,
            seed,
            [DEFAULT_HERO_POWER.to_owned(), DEFAULT_HERO_POWER.to_owned()],
            ["mage".to_owned(), "mage".to_owned()],
        )
    }

    pub fn new_with_hero_powers(
        runtime: R,
        deck_one: Vec<String>,
        deck_two: Vec<String>,
        seed: u64,
        hero_powers: [String; 2],
    ) -> Result<Self, GameError> {
        Self::new_with_hero_powers_and_classes(
            runtime,
            deck_one,
            deck_two,
            seed,
            hero_powers,
            ["mage".to_owned(), "mage".to_owned()],
        )
    }

    pub fn new_with_hero_powers_and_classes(
        runtime: R,
        deck_one: Vec<String>,
        deck_two: Vec<String>,
        seed: u64,
        hero_powers: [String; 2],
        classes: [String; 2],
    ) -> Result<Self, GameError> {
        Self::new_with_deck_class_enforcement(
            runtime,
            deck_one,
            deck_two,
            seed,
            GameSetupOptions {
                hero_powers,
                classes,
                enforce_deck_classes: [true, true],
                starting_player: PlayerId::ONE,
                sideboards: Default::default(),
            },
        )
    }

    /// Constructs a normal game with an explicitly selected first player.
    ///
    /// Callers that randomize the opening order must store their result here;
    /// replay and snapshot reconstruction will then reproduce the same order.
    pub fn new_with_hero_powers_classes_and_starting_player(
        runtime: R,
        deck_one: Vec<String>,
        deck_two: Vec<String>,
        seed: u64,
        hero_powers: [String; 2],
        classes: [String; 2],
        starting_player: PlayerId,
    ) -> Result<Self, GameError> {
        Self::new_with_deck_class_enforcement(
            runtime,
            deck_one,
            deck_two,
            seed,
            GameSetupOptions {
                hero_powers,
                classes,
                enforce_deck_classes: [true, true],
                starting_player,
                sideboards: Default::default(),
            },
        )
    }

    /// Constructs a mechanics sandbox whose decks may intentionally mix
    /// classes. Normal games should use the enforcing constructors above.
    pub fn new_unrestricted_with_hero_powers_and_classes(
        runtime: R,
        deck_one: Vec<String>,
        deck_two: Vec<String>,
        seed: u64,
        hero_powers: [String; 2],
        classes: [String; 2],
    ) -> Result<Self, GameError> {
        Self::new_with_deck_class_enforcement(
            runtime,
            deck_one,
            deck_two,
            seed,
            GameSetupOptions {
                hero_powers,
                classes,
                enforce_deck_classes: [false, false],
                starting_player: PlayerId::ONE,
                sideboards: Default::default(),
            },
        )
    }

    /// Constructs an unrestricted mechanics sandbox with an explicitly
    /// selected first player.
    pub fn new_unrestricted_with_hero_powers_classes_and_starting_player(
        runtime: R,
        deck_one: Vec<String>,
        deck_two: Vec<String>,
        seed: u64,
        hero_powers: [String; 2],
        classes: [String; 2],
        starting_player: PlayerId,
    ) -> Result<Self, GameError> {
        Self::new_with_deck_class_enforcement(
            runtime,
            deck_one,
            deck_two,
            seed,
            GameSetupOptions {
                hero_powers,
                classes,
                enforce_deck_classes: [false, false],
                starting_player,
                sideboards: Default::default(),
            },
        )
    }

    /// Constructs a game with external constructed sideboards.
    pub fn new_with_sideboards_hero_powers_classes_and_starting_player(
        runtime: R,
        deck_one: Vec<String>,
        deck_two: Vec<String>,
        sideboards: [BTreeMap<String, Vec<String>>; 2],
        seed: u64,
        hero_powers: [String; 2],
        classes: [String; 2],
        starting_player: PlayerId,
        unrestricted: bool,
    ) -> Result<Self, GameError> {
        Self::new_with_deck_class_enforcement(
            runtime,
            deck_one,
            deck_two,
            seed,
            GameSetupOptions {
                hero_powers,
                classes,
                enforce_deck_classes: [!unrestricted, !unrestricted],
                starting_player,
                sideboards,
            },
        )
    }

    fn new_with_deck_class_enforcement(
        runtime: R,
        deck_one: Vec<String>,
        deck_two: Vec<String>,
        seed: u64,
        options: GameSetupOptions,
    ) -> Result<Self, GameError> {
        let GameSetupOptions {
            hero_powers,
            classes,
            enforce_deck_classes,
            starting_player,
            sideboards,
        } = options;
        if !matches!(starting_player, PlayerId::ONE | PlayerId::TWO) {
            return Err(GameError::InvalidStartingPlayer(starting_player));
        }
        let deck_rules = [deck_one.as_slice(), deck_two.as_slice()].map(|deck| {
            let definitions = deck
                .iter()
                .filter_map(|card_id| runtime.definition(card_id));
            let maximum = definitions
                .clone()
                .filter_map(|definition| definition.deck_size)
                .map(usize::from)
                .max()
                .unwrap_or(30);
            let starting_health = definitions
                .filter_map(|definition| definition.starting_health)
                .max()
                .unwrap_or(30);
            (maximum, starting_health)
        });
        for (player, deck) in [
            (PlayerId::ONE, deck_one.as_slice()),
            (PlayerId::TWO, deck_two.as_slice()),
        ] {
            if deck.is_empty() {
                return Err(GameError::EmptyDeck(player));
            }
            let maximum = deck_rules[player.index()].0;
            if deck.len() > maximum {
                return Err(GameError::DeckTooLarge {
                    player,
                    cards: deck.len(),
                    maximum,
                });
            }
        }
        for player in [PlayerId::ONE, PlayerId::TWO] {
            let class = &classes[player.index()];
            if class.trim().is_empty() || class.len() > 64 {
                return Err(GameError::InvalidPlayerClass {
                    player,
                    class: class.clone(),
                });
            }
        }
        let mut entities = std::collections::BTreeMap::new();
        let hero_one = EntityId(1);
        let hero_two = EntityId(2);
        entities.insert(hero_one, Self::hero(hero_one, PlayerId::ONE, 1));
        entities.insert(hero_two, Self::hero(hero_two, PlayerId::TWO, 2));

        let empty_player = |id, hero, class| PlayerState {
            id,
            class,
            hero,
            deck: VecDeque::new(),
            hand: Vec::new(),
            board: Vec::new(),
            weapon: None,
            hero_power: EntityId(0),
            hero_power_used: false,
            hero_power_uses: 0,
            hero_power_uses_this_turn: 0,
            secrets: Vec::new(),
            graveyard: Vec::new(),
            minions_died_history: Vec::new(),
            discarded_cards_history: Vec::new(),
            discarded_card_ids_history: Vec::new(),
            starting_deck: Vec::new(),
            sideboards: BTreeMap::new(),
            cards_added_to_hand_history: Vec::new(),
            mana: 0,
            max_mana: 0,
            temporary_mana: 0,
            corpses: 0,
            corpses_spent: 0,
            overload_pending: 0,
            overloaded_mana: 0,
            overload_queued_total: 0,
            hero_last_healed_turn: None,
            cards_played_this_turn: 0,
            cards_played_history: Vec::new(),
            cards_played_last_turn: Vec::new(),
            cards_played_current_turn: Vec::new(),
            spells_cast_history: Vec::new(),
            spell_cast_records: Vec::new(),
            minions_played_history: Vec::new(),
            minions_summoned_history: Vec::new(),
            weapons_played_history: Vec::new(),
            weapons_destroyed_history: Vec::new(),
            locations_played_history: Vec::new(),
            fatigue: 0,
            keywords: Vec::new(),
            public_keywords: Vec::new(),
            script_data: Default::default(),
            extra_turns: 0,
        };

        let state = GameState {
            rng_seed: seed,
            random_counter: 0,
            turn: 0,
            starting_player,
            active_player: starting_player,
            players: [
                empty_player(PlayerId::ONE, hero_one, classes[0].clone()),
                empty_player(PlayerId::TWO, hero_two, classes[1].clone()),
            ],
            entities,
            next_entity_id: 3,
            next_timestamp: 3,
            next_enchantment_id: 1,
            next_event_id: 1,
            outcome: None,
            mulligan: None,
            pending_input: None,
            log: Vec::new(),
            public_logs: std::array::from_fn(|_| std::sync::Arc::new(Vec::new())),
        };

        let initial_decks = [deck_one.clone(), deck_two.clone()];
        let initial_sideboards = sideboards.clone();
        let initial_hero_powers = hero_powers.clone();
        let initial_classes = classes;
        let mut game = Self {
            runtime,
            state,
            rng: ChaCha8Rng::seed_from_u64(seed),
            initial_decks,
            initial_sideboards,
            initial_hero_powers,
            initial_classes,
            enforce_deck_classes,
            command_history: Vec::new(),
        };
        for player in [PlayerId::ONE, PlayerId::TWO] {
            let Some(hero_card_id) = default_hero_for_class(&game.state.player(player).class)
            else {
                continue;
            };
            // Small unit-test runtimes are allowed to omit cosmetic starting
            // Hero definitions. The full game pack contains all eleven.
            let Some(definition) = game.runtime.definition(hero_card_id).cloned() else {
                continue;
            };
            if definition.kind != CardKind::Hero {
                return Err(GameError::InvalidHero(hero_card_id.to_owned()));
            }
            let hero = game.state.player(player).hero;
            let timestamp = game.state.entities[&hero].timestamp;
            game.state.entities.insert(
                hero,
                Self::from_definition(hero, player, Zone::Hero, timestamp, &definition),
            );
        }
        for player in [PlayerId::ONE, PlayerId::TWO] {
            let starting_health = deck_rules[player.index()].1;
            let hero = game.state.player(player).hero;
            let entity = game.state.entities.get_mut(&hero).unwrap();
            entity.base_health = starting_health;
            entity.max_health = starting_health;
            entity.damage = 0;
        }
        for player in [PlayerId::ONE, PlayerId::TWO] {
            let definition = game
                .runtime
                .definition(&hero_powers[player.index()])
                .ok_or_else(|| GameError::UnknownCard(hero_powers[player.index()].clone()))?;
            if definition.kind != CardKind::HeroPower {
                return Err(GameError::InvalidHeroPower(
                    hero_powers[player.index()].clone(),
                ));
            }
            let hero_power =
                game.instantiate(&hero_powers[player.index()], player, Zone::HeroPower)?;
            game.state.player_mut(player).hero_power = hero_power;
        }
        game.install_deck(PlayerId::ONE, deck_one)?;
        game.install_deck(PlayerId::TWO, deck_two)?;
        game.install_sideboards(PlayerId::ONE, sideboards[0].clone())?;
        game.install_sideboards(PlayerId::TWO, sideboards[1].clone())?;

        // Start-of-game cards listen from the deck. Resolve them before the
        // opening hand is drawn, matching Hearthstone's setup ordering.
        let effects = game.publish(GameEvent::GameStarted)?;
        game.resolve_effects(effects)?;

        let second_player = starting_player.opponent();
        game.draw_starting_hand(starting_player, 3)?;
        game.draw_starting_hand(second_player, 4)?;
        game.state.mulligan = Some(crate::MulliganState {
            current_player: starting_player,
            eligible: [
                game.state.player(PlayerId::ONE).hand.clone(),
                game.state.player(PlayerId::TWO).hand.clone(),
            ],
        });
        game.state.validate().map_err(GameError::Invariant)?;
        Ok(game)
    }

    pub fn from_replay(runtime: R, replay: &Replay) -> Result<Self, GameError> {
        if replay.format_version != 3 {
            return Err(GameError::ReplayCommandFailed {
                index: 0,
                message: format!("unsupported replay format {}", replay.format_version),
            });
        }
        if runtime.pack_hash() != replay.card_pack_hash {
            return Err(GameError::ReplayPackMismatch {
                replay: replay.card_pack_hash.clone(),
                loaded: runtime.pack_hash().to_owned(),
            });
        }
        let mut game = Self::new_with_deck_class_enforcement(
            runtime,
            replay.decks[0].clone(),
            replay.decks[1].clone(),
            replay.seed,
            GameSetupOptions {
                hero_powers: replay.hero_powers.clone(),
                classes: replay.classes.clone(),
                enforce_deck_classes: replay.enforce_deck_classes,
                starting_player: replay.starting_player,
                sideboards: replay.sideboards.clone(),
            },
        )?;
        for (index, command) in replay.commands.iter().cloned().enumerate() {
            game.dispatch(command)
                .map_err(|error| GameError::ReplayCommandFailed {
                    index,
                    message: error.to_string(),
                })?;
        }
        Ok(game)
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn runtime(&self) -> &R {
        &self.runtime
    }

    /// Consumes the game and returns its rule runtime. Frontends that run many
    /// matches can reuse an expensive runtime without teaching the rules engine
    /// about worker pools, reinforcement learning, or any other caller policy.
    pub fn into_runtime(self) -> R {
        self.runtime
    }

    pub fn replay(&self) -> Replay {
        Replay {
            format_version: 3,
            card_pack_hash: self.runtime.pack_hash().to_owned(),
            seed: self.state.rng_seed,
            starting_player: self.state.starting_player,
            decks: self.initial_decks.clone(),
            sideboards: self.initial_sideboards.clone(),
            hero_powers: self.initial_hero_powers.clone(),
            classes: self.initial_classes.clone(),
            enforce_deck_classes: self.enforce_deck_classes,
            commands: self.command_history.clone(),
        }
    }

    /// Creates a portable checkpoint. The embedded replay is used as a proof when restoring,
    /// so corrupted or hand-edited authoritative state is rejected rather than trusted.
    pub fn snapshot(&self) -> GameSnapshot {
        GameSnapshot {
            format_version: 3,
            replay: self.replay(),
            state: self.state.clone(),
        }
    }

    pub fn from_snapshot(runtime: R, snapshot: &GameSnapshot) -> Result<Self, GameError> {
        if snapshot.format_version != 3 {
            return Err(GameError::UnsupportedSnapshot(snapshot.format_version));
        }
        let game = Self::from_replay(runtime, &snapshot.replay)?;
        let mut expected = snapshot.state.clone();
        // Public logs are a replay-derived observer cache and are deliberately
        // omitted from serialized snapshots. Never trust them as proof of the
        // authoritative state.
        expected.public_logs = game.state.public_logs.clone();
        if game.state != expected {
            return Err(GameError::SnapshotStateMismatch);
        }
        Ok(game)
    }
}
