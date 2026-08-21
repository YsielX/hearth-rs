use super::*;

pub(super) struct CardCreation {
    pub source: EntityId,
    pub player: PlayerId,
    pub card_id: String,
    pub destination: ZonePlacement,
    pub position: Option<usize>,
    pub base_attack: Option<i32>,
    pub base_health: Option<i32>,
    pub base_cost: Option<u8>,
    pub base_spell_damage: Option<i32>,
    pub keywords: Option<Vec<String>>,
    pub attached_scripts: Vec<String>,
}

impl<R: CardRuntime> Game<R> {
    pub(super) fn create_card_from_spec(
        &mut self,
        creation: CardCreation,
    ) -> Result<Vec<GameEvent>, GameError> {
        for attached in &creation.attached_scripts {
            self.runtime
                .definition(attached)
                .ok_or_else(|| GameError::UnknownCard(attached.clone()))?;
        }
        let actual_destination = if creation.destination == ZonePlacement::Hand
            && self.state.player(creation.player).hand.len() >= MAX_HAND_SIZE
        {
            ZonePlacement::Graveyard
        } else {
            creation.destination
        };
        if matches!(
            actual_destination,
            ZonePlacement::Board | ZonePlacement::Secret
        ) {
            return Err(GameError::Invariant(
                "create_card supports hidden and terminal zones; use summon or play-secret semantics for active zones"
                    .to_owned(),
            ));
        }
        let card = self.instantiate(
            &creation.card_id,
            creation.player,
            actual_destination.zone(),
        )?;
        {
            let entity = self.state.entities.get_mut(&card).unwrap();
            entity.base_attack = creation.base_attack.unwrap_or(entity.base_attack);
            entity.base_health = creation.base_health.unwrap_or(entity.base_health).max(1);
            entity.base_cost = creation.base_cost.unwrap_or(entity.base_cost);
            entity.base_spell_damage = creation
                .base_spell_damage
                .unwrap_or(entity.base_spell_damage)
                .max(0);
            if let Some(keywords) = creation.keywords {
                entity.base_keywords = keywords;
            }
            entity.base_attached_cards = creation.attached_scripts.clone();
            entity.attached_cards = creation.attached_scripts;
            Self::recompute_entity(entity);
        }
        self.install_created_card(card, creation.player, actual_destination, creation.position);
        if creation.destination == ZonePlacement::Hand
            && actual_destination == ZonePlacement::Graveyard
        {
            Ok(vec![GameEvent::CardBurned {
                player: creation.player,
                card,
                source: Some(creation.source),
            }])
        } else {
            Ok(vec![GameEvent::CardCreated {
                source: creation.source,
                player: creation.player,
                card,
            }])
        }
    }

    fn install_created_card(
        &mut self,
        card: EntityId,
        player: PlayerId,
        destination: ZonePlacement,
        position: Option<usize>,
    ) {
        match destination {
            ZonePlacement::Hand => {
                self.state
                    .entities
                    .get_mut(&card)
                    .unwrap()
                    .entered_hand_turn = Some(self.state.turn);
                let position = position
                    .unwrap_or(self.state.player(player).hand.len())
                    .min(self.state.player(player).hand.len());
                self.state.player_mut(player).hand.insert(position, card);
            }
            ZonePlacement::DeckTop => self.state.player_mut(player).deck.push_front(card),
            ZonePlacement::DeckBottom => self.state.player_mut(player).deck.push_back(card),
            ZonePlacement::DeckRandom => {
                let len = self.state.player(player).deck.len();
                let position = self.rng.random_range(0..=len);
                self.state.random_counter = self.state.random_counter.saturating_add(1);
                self.state.player_mut(player).deck.insert(position, card);
            }
            ZonePlacement::Graveyard => self.state.player_mut(player).graveyard.push(card),
            ZonePlacement::Removed => {}
            ZonePlacement::Board | ZonePlacement::Secret => unreachable!(),
        }
    }
}
