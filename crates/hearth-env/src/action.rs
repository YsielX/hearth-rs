use hearth_core::{EntityId, LegalAction, PlayerCommand, PlayerView};
use serde::{Deserialize, Serialize};

use crate::entity_refs::EpisodeRefs;
use crate::{EntityRef, EnvError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Mulligan,
    PlayCard,
    PlayCardAt,
    TradeCard,
    UseCardAction,
    Attack,
    UseHeroPower,
    UseLocation,
    EndTurn,
    Concede,
    Choose,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionObservation {
    /// Index accepted by `HearthEnv::step` for this decision only.
    pub index: u32,
    pub kind: ActionKind,
    pub sources: Vec<EntityRef>,
    pub target: Option<EntityRef>,
    pub board_position: Option<u8>,
    pub mana_cost: u8,
    pub card_action: Option<String>,
    pub semantic_card_id: Option<String>,
    pub choice_index: Option<u16>,
}

pub(crate) fn encode_action(
    index: usize,
    action: &LegalAction,
    refs: &EpisodeRefs,
    view: &PlayerView,
) -> Result<ActionObservation, EnvError> {
    let map = |entity: EntityId| refs.get(entity);
    let mut encoded = ActionObservation {
        index: u32::try_from(index).map_err(|_| EnvError::PublicEventValueTooLarge)?,
        kind: ActionKind::EndTurn,
        sources: Vec::new(),
        target: None,
        board_position: None,
        mana_cost: action.mana_cost,
        card_action: None,
        semantic_card_id: action.semantic_card_id.clone(),
        choice_index: None,
    };
    match &action.command {
        PlayerCommand::Mulligan { replace } => {
            encoded.kind = ActionKind::Mulligan;
            encoded.sources = replace.iter().copied().map(map).collect::<Result<_, _>>()?;
        }
        PlayerCommand::PlayCard { card, target } => {
            encoded.kind = ActionKind::PlayCard;
            encoded.sources.push(map(*card)?);
            encoded.target = target.map(map).transpose()?;
        }
        PlayerCommand::PlayCardAt {
            card,
            target,
            position,
        } => {
            encoded.kind = ActionKind::PlayCardAt;
            encoded.sources.push(map(*card)?);
            encoded.target = target.map(map).transpose()?;
            encoded.board_position =
                Some(u8::try_from(*position).map_err(|_| EnvError::PublicEventValueTooLarge)?);
        }
        PlayerCommand::TradeCard { card } => {
            encoded.kind = ActionKind::TradeCard;
            encoded.sources.push(map(*card)?);
        }
        PlayerCommand::UseCardAction {
            card,
            action,
            target,
        } => {
            encoded.kind = ActionKind::UseCardAction;
            encoded.sources.push(map(*card)?);
            encoded.target = target.map(map).transpose()?;
            encoded.card_action = Some(action.clone());
        }
        PlayerCommand::Attack { attacker, defender } => {
            encoded.kind = ActionKind::Attack;
            encoded.sources.push(map(*attacker)?);
            encoded.target = Some(map(*defender)?);
        }
        PlayerCommand::UseHeroPower { target } => {
            encoded.kind = ActionKind::UseHeroPower;
            encoded
                .sources
                .push(map(view.player(view.viewer).hero_power)?);
            encoded.target = target.map(map).transpose()?;
        }
        PlayerCommand::UseLocation { location, target } => {
            encoded.kind = ActionKind::UseLocation;
            encoded.sources.push(map(*location)?);
            encoded.target = target.map(map).transpose()?;
        }
        PlayerCommand::EndTurn => encoded.kind = ActionKind::EndTurn,
        PlayerCommand::Concede | PlayerCommand::ConcedePlayer { .. } => {
            encoded.kind = ActionKind::Concede;
        }
        PlayerCommand::Choose { index } => {
            encoded.kind = ActionKind::Choose;
            encoded.choice_index =
                Some(u16::try_from(*index).map_err(|_| EnvError::PublicEventValueTooLarge)?);
        }
    }
    Ok(encoded)
}
