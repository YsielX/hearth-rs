use super::*;

impl<R: CardRuntime> Game<R> {
    pub(super) fn exchange_zone_contents(
        &mut self,
        first: PlayerId,
        second: PlayerId,
        zone: Zone,
    ) -> Result<Vec<GameEvent>, GameError> {
        if first == second {
            return Ok(Vec::new());
        }
        let (first_cards, second_cards) = match zone {
            Zone::Deck => {
                let first_cards = std::mem::take(&mut self.state.player_mut(first).deck);
                let second_cards =
                    std::mem::replace(&mut self.state.player_mut(second).deck, first_cards);
                self.state.player_mut(first).deck = second_cards;
                (
                    self.state.player(first).deck.iter().copied().collect(),
                    self.state.player(second).deck.iter().copied().collect(),
                )
            }
            Zone::Hand => {
                let first_cards = std::mem::take(&mut self.state.player_mut(first).hand);
                let second_cards =
                    std::mem::replace(&mut self.state.player_mut(second).hand, first_cards);
                self.state.player_mut(first).hand = second_cards;
                (
                    self.state.player(first).hand.clone(),
                    self.state.player(second).hand.clone(),
                )
            }
            Zone::Graveyard => {
                let first_cards = std::mem::take(&mut self.state.player_mut(first).graveyard);
                let second_cards =
                    std::mem::replace(&mut self.state.player_mut(second).graveyard, first_cards);
                self.state.player_mut(first).graveyard = second_cards;
                (
                    self.state.player(first).graveyard.clone(),
                    self.state.player(second).graveyard.clone(),
                )
            }
            _ => {
                return Err(GameError::Invariant(format!(
                    "cannot exchange ownership of {zone:?} contents"
                )));
            }
        };
        for (player, cards) in [(first, first_cards), (second, second_cards)] {
            for card in cards {
                let entity = self.state.entities.get_mut(&card).unwrap();
                entity.owner = player;
                entity.controller = player;
            }
        }
        self.refresh_auras()?;
        Ok(Vec::new())
    }
}
