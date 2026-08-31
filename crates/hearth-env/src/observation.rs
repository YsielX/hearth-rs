use std::collections::{BTreeMap, BTreeSet};

use hearth_core::{
    CardKind, ChoiceOptionValueView, EntityId, PendingInputView, PlayerId, PlayerView,
};
use serde::{Deserialize, Serialize};

use crate::entity_refs::EpisodeRefs;
use crate::history::{EventWindow, PublicHistory, ViewerMemory};
use crate::{EnvError, OBSERVATION_SCHEMA_VERSION};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelativePlayer {
    SelfPlayer,
    Opponent,
}

impl RelativePlayer {
    pub(crate) fn from_player(player: PlayerId, viewer: PlayerId) -> Self {
        if player == viewer {
            Self::SelfPlayer
        } else {
            Self::Opponent
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionPhase {
    Mulligan,
    Choice,
    Main,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EntityRef(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityArea {
    Hero,
    HeroPower,
    Weapon,
    Board,
    Hand,
    Secret,
    PublicObjective,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityObservation {
    pub entity: EntityRef,
    pub card_id: String,
    pub kind: CardKind,
    pub owner: RelativePlayer,
    pub controller: RelativePlayer,
    pub area: EntityArea,
    pub position: u8,
    pub attack: i32,
    pub max_health: i32,
    pub damage: i32,
    pub armor: i32,
    pub cost: u8,
    pub spell_damage: i32,
    pub exhausted: bool,
    pub frozen: bool,
    pub attacks_this_turn: u8,
    pub location_cooldown: u8,
    pub keywords: Vec<String>,
    pub silenced: bool,
    pub public_cards: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerObservation {
    pub class: String,
    pub hero: EntityRef,
    pub hero_power: EntityRef,
    pub weapon: Option<EntityRef>,
    pub board: Vec<EntityRef>,
    /// Empty for the opponent.
    pub hand: Vec<EntityRef>,
    /// Ordinary Secret identities are present only for the observing player.
    pub secrets: Vec<EntityRef>,
    pub public_objectives: Vec<EntityRef>,
    pub deck_size: u8,
    pub hand_size: u8,
    pub secrets_count: u8,
    pub mana: u8,
    pub max_mana: u8,
    pub temporary_mana: u8,
    pub resources: BTreeMap<String, u32>,
    pub resources_spent: BTreeMap<String, u32>,
    pub public_statuses: Vec<String>,
    pub overload_pending: u8,
    pub overloaded_mana: u8,
    pub fatigue: u32,
    pub hero_power_used: bool,
    pub hero_power_uses_this_turn: u8,
    pub cards_played_this_turn: u32,
    pub history: PublicHistory,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChoiceOptionValueObservation {
    Entity { entity: EntityRef, card_id: String },
    Card { card_id: String },
    Opaque,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceOptionObservation {
    pub label: String,
    pub value: ChoiceOptionValueObservation,
    pub semantic_card_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceObservation {
    pub prompt: String,
    pub options: Vec<ChoiceOptionObservation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub schema_version: u32,
    /// Physical seat is metadata, never an input for ownership decisions.
    pub seat: u8,
    pub turn: u32,
    pub active_player: RelativePlayer,
    pub phase: DecisionPhase,
    pub self_player: PlayerObservation,
    pub opponent: PlayerObservation,
    pub entities: Vec<EntityObservation>,
    pub history: EventWindow,
    pub mulligan_eligible: Vec<EntityRef>,
    pub pending_choice: Option<ChoiceObservation>,
}

fn encode_choice(
    input: &PendingInputView,
    refs: &mut EpisodeRefs,
) -> Result<ChoiceObservation, EnvError> {
    let options = input
        .options
        .iter()
        .map(|option| {
            let value = match &option.value {
                ChoiceOptionValueView::Entity(entity) => ChoiceOptionValueObservation::Entity {
                    entity: refs.observe_public(entity)?,
                    card_id: entity.card_id.clone(),
                },
                ChoiceOptionValueView::Card(card_id) => ChoiceOptionValueObservation::Card {
                    card_id: card_id.clone(),
                },
                ChoiceOptionValueView::Opaque => ChoiceOptionValueObservation::Opaque,
            };
            Ok(ChoiceOptionObservation {
                label: option.label.clone(),
                value,
                semantic_card_ids: option.semantic_card_ids.clone(),
            })
        })
        .collect::<Result<Vec<_>, EnvError>>()?;
    Ok(ChoiceObservation {
        prompt: input.prompt.clone(),
        options,
    })
}

pub(crate) fn build_observation(
    view: &PlayerView,
    memory: &mut ViewerMemory,
    history_limit: Option<usize>,
) -> Result<Observation, EnvError> {
    let self_id = view.viewer;
    let opponent_id = self_id.opponent();
    let histories = memory.histories().clone();
    let history = memory.window(history_limit);
    let refs = &mut memory.refs;
    let mut entities = Vec::new();
    let mut current_entities = BTreeSet::new();

    let mut add =
        |id: EntityId, area: EntityArea, position: usize| -> Result<EntityRef, EnvError> {
            if !current_entities.insert(id) {
                return refs.get(id);
            }
            let reference = refs.observe(id)?;
            let entity = view.entity(id).ok_or(EnvError::HiddenActionEntity(id))?;
            entities.push(EntityObservation {
                entity: reference,
                card_id: entity.card_id.clone(),
                kind: entity.kind,
                owner: RelativePlayer::from_player(entity.owner, self_id),
                controller: RelativePlayer::from_player(entity.controller, self_id),
                area,
                position: u8::try_from(position).map_err(|_| EnvError::PublicEventValueTooLarge)?,
                attack: entity.attack,
                max_health: entity.max_health,
                damage: entity.damage,
                armor: entity.armor,
                cost: entity.cost,
                spell_damage: entity.spell_damage,
                exhausted: entity.exhausted,
                frozen: entity.frozen,
                attacks_this_turn: entity.attacks_this_turn,
                location_cooldown: entity.location_cooldown,
                keywords: entity.keywords.clone(),
                silenced: entity.silenced,
                public_cards: entity.public_cards.clone(),
            });
            Ok(reference)
        };

    for player_id in [self_id, opponent_id] {
        let player = view.player(player_id);
        add(player.hero, EntityArea::Hero, 0)?;
        add(player.hero_power, EntityArea::HeroPower, 0)?;
        if let Some(weapon) = player.weapon {
            add(weapon, EntityArea::Weapon, 0)?;
        }
        for (position, entity) in player.board.iter().copied().enumerate() {
            add(entity, EntityArea::Board, position)?;
        }
        for (position, entity) in player.public_objectives.iter().copied().enumerate() {
            add(entity, EntityArea::PublicObjective, position)?;
        }
    }
    for (position, entity) in view.player(self_id).hand.iter().copied().enumerate() {
        add(entity, EntityArea::Hand, position)?;
    }
    for (position, entity) in view.player(self_id).secrets.iter().copied().enumerate() {
        add(entity, EntityArea::Secret, position)?;
    }
    drop(add);

    let pending_choice = view
        .pending_input
        .as_ref()
        .map(|input| encode_choice(input, refs))
        .transpose()?;

    let player_observation = |player_id: PlayerId| -> Result<PlayerObservation, EnvError> {
        let player = view.player(player_id);
        let map = |id: EntityId| refs.get(id);
        Ok(PlayerObservation {
            class: player.class.clone(),
            hero: map(player.hero)?,
            hero_power: map(player.hero_power)?,
            weapon: player.weapon.map(map).transpose()?,
            board: player
                .board
                .iter()
                .copied()
                .map(map)
                .collect::<Result<_, _>>()?,
            hand: player
                .hand
                .iter()
                .copied()
                .map(map)
                .collect::<Result<_, _>>()?,
            secrets: player
                .secrets
                .iter()
                .copied()
                .filter(|entity| !player.public_objectives.contains(entity))
                .map(map)
                .collect::<Result<_, _>>()?,
            public_objectives: player
                .public_objectives
                .iter()
                .copied()
                .map(map)
                .collect::<Result<_, _>>()?,
            deck_size: player.deck_size.min(usize::from(u8::MAX)) as u8,
            hand_size: player.hand_size.min(usize::from(u8::MAX)) as u8,
            secrets_count: player.secrets_count.min(usize::from(u8::MAX)) as u8,
            mana: player.mana,
            max_mana: player.max_mana,
            temporary_mana: player.temporary_mana,
            resources: player.resources.clone(),
            resources_spent: player.resources_spent.clone(),
            public_statuses: player.public_statuses.clone(),
            overload_pending: player.overload_pending,
            overloaded_mana: player.overloaded_mana,
            fatigue: player.fatigue,
            hero_power_used: player.hero_power_used,
            hero_power_uses_this_turn: player.hero_power_uses_this_turn,
            cards_played_this_turn: player.cards_played_this_turn,
            history: histories[player_id.index()].clone(),
        })
    };

    let phase = if !view.mulligan_eligible.is_empty() {
        DecisionPhase::Mulligan
    } else if view.pending_input.is_some() {
        DecisionPhase::Choice
    } else {
        DecisionPhase::Main
    };
    let mulligan_eligible = view
        .mulligan_eligible
        .iter()
        .map(|entity| refs.get(*entity))
        .collect::<Result<Vec<_>, _>>()?;
    let observation = Observation {
        schema_version: OBSERVATION_SCHEMA_VERSION,
        seat: self_id.0,
        turn: view.turn,
        active_player: RelativePlayer::from_player(view.active_player, self_id),
        phase,
        self_player: player_observation(self_id)?,
        opponent: player_observation(opponent_id)?,
        entities,
        history,
        mulligan_eligible,
        pending_choice,
    };
    Ok(observation)
}

#[cfg(test)]
mod tests {
    use hearth_core::{
        ChoiceOptionValueView, ChoiceOptionView, EntityId, PendingInputView, PublicEntity,
    };

    use super::*;

    #[test]
    fn choice_encoding_preserves_public_structure_and_hides_payloads() {
        let input = PendingInputView {
            prompt: "Choose".to_owned(),
            options: vec![
                ChoiceOptionView {
                    label: "Entity".to_owned(),
                    value: ChoiceOptionValueView::Entity(PublicEntity {
                        id: EntityId(99),
                        card_id: "ENTITY_CARD".to_owned(),
                    }),
                    semantic_card_ids: vec!["ENTITY_SEMANTIC".to_owned()],
                },
                ChoiceOptionView {
                    label: "Card".to_owned(),
                    value: ChoiceOptionValueView::Card("CARD".to_owned()),
                    semantic_card_ids: vec!["CARD".to_owned()],
                },
                ChoiceOptionView {
                    label: "Secret payload".to_owned(),
                    value: ChoiceOptionValueView::Opaque,
                    semantic_card_ids: Vec::new(),
                },
            ],
        };
        let choice = encode_choice(&input, &mut EpisodeRefs::default()).unwrap();
        assert_eq!(choice.prompt, "Choose");
        assert!(matches!(
            &choice.options[0].value,
            ChoiceOptionValueObservation::Entity {
                entity: EntityRef(0),
                card_id
            } if card_id == "ENTITY_CARD"
        ));
        assert_eq!(choice.options[0].semantic_card_ids, ["ENTITY_SEMANTIC"]);
        assert!(matches!(
            &choice.options[1].value,
            ChoiceOptionValueObservation::Card { card_id } if card_id == "CARD"
        ));
        assert_eq!(
            choice.options[2].value,
            ChoiceOptionValueObservation::Opaque
        );
    }
}
