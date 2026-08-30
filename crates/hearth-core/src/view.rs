use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use crate::{
    CardId, CardKind, ChoiceValue, Entity, EntityId, GameOutcome, GameState, PlayerId,
    PublicEntity, PublicEventRecord, Zone,
};

/// An entity projection containing only information visible to one player.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityView {
    pub id: EntityId,
    pub card_id: CardId,
    pub kind: CardKind,
    pub owner: PlayerId,
    pub controller: PlayerId,
    pub zone: Zone,
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
}

impl EntityView {
    fn from_entity(entity: &Entity) -> Self {
        Self {
            id: entity.id,
            card_id: entity.card_id.clone(),
            kind: entity.kind,
            owner: entity.owner,
            controller: entity.controller,
            zone: entity.zone,
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
        }
    }

    pub fn health(&self) -> i32 {
        self.max_health - self.damage
    }

    pub fn has_keyword(&self, keyword: &str) -> bool {
        self.keywords.iter().any(|candidate| candidate == keyword)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerStateView {
    pub id: PlayerId,
    pub class: String,
    pub hero: EntityId,
    pub deck_size: usize,
    pub hand_size: usize,
    /// Populated only for the viewing player.
    pub hand: Vec<EntityId>,
    /// Remaining constructed sideboards, populated only for the viewing player.
    pub sideboards: BTreeMap<CardId, Vec<CardId>>,
    pub board: Vec<EntityId>,
    pub weapon: Option<EntityId>,
    pub hero_power: EntityId,
    pub hero_power_used: bool,
    pub hero_power_uses_this_turn: u8,
    /// Count of unrevealed ordinary Secrets. Public Quests are excluded.
    pub secrets_count: usize,
    /// Populated only for the viewing player.
    pub secrets: Vec<EntityId>,
    /// Quests, Questlines, and Sidequests are visible to both players.
    pub public_objectives: Vec<EntityId>,
    pub mana: u8,
    pub max_mana: u8,
    pub temporary_mana: u8,
    /// Public script-defined resource balances.
    pub resources: BTreeMap<String, u32>,
    /// Public lifetime resource spending totals.
    pub resources_spent: BTreeMap<String, u32>,
    /// Public, persistent status labels, independent of executable rules.
    pub public_statuses: Vec<String>,
    pub overload_pending: u8,
    pub overloaded_mana: u8,
    pub fatigue: u32,
    pub cards_played_this_turn: u32,
}

impl PlayerStateView {
    pub fn resource(&self, resource: &str) -> u32 {
        self.resources.get(resource).copied().unwrap_or_default()
    }

    pub fn resource_spent(&self, resource: &str) -> u32 {
        self.resources_spent
            .get(resource)
            .copied()
            .unwrap_or_default()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChoiceOptionValueView {
    /// An entity explicitly revealed by this choice, including choices from a
    /// normally hidden zone such as Dredge.
    Entity(PublicEntity),
    /// A card definition explicitly offered by this choice, such as Discover.
    Card(CardId),
    /// The internal continuation payload is deliberately not player-visible.
    Opaque,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChoiceOptionView {
    pub label: String,
    pub value: ChoiceOptionValueView,
}

impl fmt::Display for ChoiceOptionView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.label)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingInputView {
    pub prompt: String,
    pub options: Vec<ChoiceOptionView>,
}

/// A stable player-facing projection. It intentionally contains no deck order,
/// opponent hand identities, opponent Secret identities, script data, aura
/// sources, RNG state, replay, or authoritative event queue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerView {
    pub viewer: PlayerId,
    pub turn: u32,
    pub active_player: PlayerId,
    pub input_player: PlayerId,
    pub players: [PlayerStateView; 2],
    pub entities: BTreeMap<EntityId, EntityView>,
    pub outcome: Option<GameOutcome>,
    pub mulligan_eligible: Vec<EntityId>,
    pub pending_input: Option<PendingInputView>,
    /// Chronological, viewer-safe history. It is projected when each internal
    /// event commits, so later transformations and hidden-zone moves cannot
    /// retroactively change what the viewer observed.
    pub history: Arc<Vec<PublicEventRecord>>,
}

impl PlayerView {
    pub fn player(&self, player: PlayerId) -> &PlayerStateView {
        &self.players[player.index()]
    }

    pub fn entity(&self, entity: EntityId) -> Option<&EntityView> {
        self.entities.get(&entity)
    }

    pub fn hero(&self, player: PlayerId) -> &EntityView {
        &self.entities[&self.player(player).hero]
    }
}

impl GameState {
    pub fn input_player(&self) -> PlayerId {
        self.mulligan
            .as_ref()
            .map(|mulligan| mulligan.current_player)
            .or_else(|| self.pending_input.as_ref().map(|pending| pending.player))
            .unwrap_or(self.active_player)
    }

    pub fn player_view(&self, viewer: PlayerId) -> PlayerView {
        let build_player = |player_id: PlayerId| {
            let player = self.player(player_id);
            let public_objectives = player
                .secrets
                .iter()
                .copied()
                .filter(|entity| {
                    self.entity(*entity)
                        .is_some_and(Entity::is_public_objective)
                })
                .collect::<Vec<_>>();
            PlayerStateView {
                id: player.id,
                class: player.class.clone(),
                hero: player.hero,
                deck_size: player.deck.len(),
                hand_size: player.hand.len(),
                hand: (player_id == viewer)
                    .then(|| player.hand.clone())
                    .unwrap_or_default(),
                sideboards: (player_id == viewer)
                    .then(|| player.sideboards.clone())
                    .unwrap_or_default(),
                board: player.board.clone(),
                weapon: player.weapon,
                hero_power: player.hero_power,
                hero_power_used: player.hero_power_used,
                hero_power_uses_this_turn: player.hero_power_uses_this_turn,
                secrets_count: player.secrets.len().saturating_sub(public_objectives.len()),
                secrets: (player_id == viewer)
                    .then(|| player.secrets.clone())
                    .unwrap_or_default(),
                public_objectives,
                mana: player.mana,
                max_mana: player.max_mana,
                temporary_mana: player.temporary_mana,
                resources: player.resources.clone(),
                resources_spent: player.resources_spent.clone(),
                public_statuses: player.public_statuses.clone(),
                overload_pending: player.overload_pending,
                overloaded_mana: player.overloaded_mana,
                fatigue: player.fatigue,
                cards_played_this_turn: player.cards_played_this_turn,
            }
        };
        let players = [build_player(PlayerId::ONE), build_player(PlayerId::TWO)];
        let mut visible = Vec::new();
        for player in &players {
            visible.extend([player.hero, player.hero_power]);
            visible.extend(player.board.iter().copied());
            visible.extend(player.weapon);
            visible.extend(player.public_objectives.iter().copied());
        }
        visible.extend(players[viewer.index()].hand.iter().copied());
        visible.extend(players[viewer.index()].secrets.iter().copied());
        visible.sort_unstable();
        visible.dedup();
        let entities = visible
            .into_iter()
            .filter_map(|id| {
                self.entity(id)
                    .map(|entity| (id, EntityView::from_entity(entity)))
            })
            .collect();
        let mulligan_eligible = self
            .mulligan
            .as_ref()
            .filter(|mulligan| mulligan.current_player == viewer)
            .map(|mulligan| mulligan.eligible[viewer.index()].clone())
            .unwrap_or_default();
        let pending_input = self
            .pending_input
            .as_ref()
            .filter(|pending| pending.player == viewer)
            .map(|pending| PendingInputView {
                prompt: pending.prompt.clone(),
                options: pending
                    .options
                    .iter()
                    .map(|option| ChoiceOptionView {
                        label: option.label.clone(),
                        value: match &option.value {
                            ChoiceValue::Entity(entity_id) => self
                                .entity(*entity_id)
                                .map(|entity| {
                                    ChoiceOptionValueView::Entity(PublicEntity {
                                        id: *entity_id,
                                        card_id: entity.card_id.clone(),
                                    })
                                })
                                .unwrap_or(ChoiceOptionValueView::Opaque),
                            ChoiceValue::Card(card_id) => {
                                ChoiceOptionValueView::Card(card_id.clone())
                            }
                            ChoiceValue::Number(_)
                            | ChoiceValue::Integer(_)
                            | ChoiceValue::Nil
                            | ChoiceValue::Boolean(_)
                            | ChoiceValue::Text(_)
                            | ChoiceValue::List(_)
                            | ChoiceValue::Object(_) => ChoiceOptionValueView::Opaque,
                        },
                    })
                    .collect(),
            });
        PlayerView {
            viewer,
            turn: self.turn,
            active_player: self.active_player,
            input_player: self.input_player(),
            players,
            entities,
            outcome: self.outcome,
            mulligan_eligible,
            pending_input,
            history: self.public_logs[viewer.index()].clone(),
        }
    }
}
