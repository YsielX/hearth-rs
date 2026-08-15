use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{CardId, Entity, EntityId, GameEvent, GameOutcome, GameState, PlayerId, Zone};

/// An entity identity frozen when a player-observable event occurs. `card_id`
/// must not be looked up later because the same entity may transform or enter a
/// hidden zone after the event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicEntity {
    pub id: EntityId,
    pub card_id: CardId,
}

/// A whitelist of game facts visible to one player. Internal script state,
/// hidden random samples, deck order, and unknown card identities have no
/// representation here, so consumers cannot accidentally serialize them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicEvent {
    GameStarted,
    TurnStarted {
        player: PlayerId,
        turn: u32,
    },
    CardDrawn {
        player: PlayerId,
        card: Option<PublicEntity>,
        source: Option<PublicEntity>,
    },
    CardBurned {
        player: PlayerId,
        card: PublicEntity,
        source: Option<PublicEntity>,
    },
    CardCreated {
        player: PlayerId,
        card: Option<PublicEntity>,
        source: Option<PublicEntity>,
    },
    Fatigue {
        player: PlayerId,
        amount: u32,
    },
    CardPlayed {
        player: PlayerId,
        card: PublicEntity,
        cost: u8,
    },
    SpellCast {
        player: PlayerId,
        spell: PublicEntity,
        generated_by: Option<PublicEntity>,
        target: Option<PublicEntity>,
        cost: u8,
    },
    SpellTargeted {
        player: PlayerId,
        spell: PublicEntity,
        target: PublicEntity,
        generated_by: Option<PublicEntity>,
    },
    MinionPlayed {
        player: PlayerId,
        minion: PublicEntity,
    },
    WeaponPlayed {
        player: PlayerId,
        weapon: PublicEntity,
    },
    LocationPlayed {
        player: PlayerId,
        location: PublicEntity,
    },
    CardCountered {
        player: PlayerId,
        card: PublicEntity,
    },
    CardDiscarded {
        player: PlayerId,
        card: PublicEntity,
        source: Option<PublicEntity>,
    },
    CardTraded {
        player: PlayerId,
        card: Option<PublicEntity>,
    },
    TradeDraw {
        player: PlayerId,
    },
    MinionSummoned {
        player: PlayerId,
        entity: PublicEntity,
    },
    Magnetized {
        player: PlayerId,
        attachment: PublicEntity,
        target: PublicEntity,
    },
    WeaponEquipped {
        player: PlayerId,
        weapon: PublicEntity,
    },
    WeaponDestroyed {
        player: PlayerId,
        weapon: PublicEntity,
    },
    LocationUsed {
        player: PlayerId,
        location: PublicEntity,
        target: Option<PublicEntity>,
    },
    LocationDestroyed {
        player: PlayerId,
        location: PublicEntity,
    },
    HeroPowerUsed {
        player: PlayerId,
        hero_power: PublicEntity,
        target: Option<PublicEntity>,
    },
    HeroPowerReplaced {
        player: PlayerId,
        source: PublicEntity,
        old: PublicEntity,
        new: PublicEntity,
    },
    HeroReplaced {
        player: PlayerId,
        old: PublicEntity,
        new: PublicEntity,
    },
    SecretPlayed {
        player: PlayerId,
        secret: Option<PublicEntity>,
    },
    SecretRevealed {
        player: PlayerId,
        secret: PublicEntity,
    },
    ZoneChanged {
        entity: PublicEntity,
        from: Zone,
        to: Zone,
    },
    ControllerChanged {
        source: Option<PublicEntity>,
        entity: PublicEntity,
        from: PlayerId,
        to: PlayerId,
    },
    Transformed {
        source: Option<PublicEntity>,
        entity: PublicEntity,
        from_card: CardId,
        to_card: CardId,
    },
    Attack {
        attacker: PublicEntity,
        defender: PublicEntity,
        collateral: Vec<(PublicEntity, i32)>,
    },
    Damaged {
        source: Option<PublicEntity>,
        target: PublicEntity,
        amount: i32,
    },
    DamagePrevented {
        source: Option<PublicEntity>,
        target: PublicEntity,
        reason: String,
    },
    Healed {
        source: Option<PublicEntity>,
        target: PublicEntity,
        amount: i32,
    },
    ArmorGained {
        source: Option<PublicEntity>,
        target: PublicEntity,
        amount: i32,
    },
    OverloadQueued {
        player: PlayerId,
        source: Option<PublicEntity>,
        amount: u8,
    },
    ManaLocked {
        player: PlayerId,
        amount: u8,
    },
    ManaUnlocked {
        player: PlayerId,
        source: Option<PublicEntity>,
        amount: u8,
    },
    OverloadCleared {
        player: PlayerId,
        source: Option<PublicEntity>,
        pending: u8,
        locked: u8,
    },
    TemporaryManaGained {
        player: PlayerId,
        source: Option<PublicEntity>,
        amount: u8,
    },
    TemporaryManaExpired {
        player: PlayerId,
        amount: u8,
    },
    ManaCrystalsGained {
        player: PlayerId,
        source: Option<PublicEntity>,
        amount: u8,
        filled: bool,
    },
    ManaCrystalsDestroyed {
        player: PlayerId,
        source: Option<PublicEntity>,
        amount: u8,
    },
    ManaSpent {
        player: PlayerId,
        source: Option<PublicEntity>,
        amount: u8,
        temporary: u8,
    },
    KeywordDisabled {
        source: Option<PublicEntity>,
        target: PublicEntity,
        keyword: String,
    },
    Frozen {
        source: Option<PublicEntity>,
        target: PublicEntity,
    },
    EntityDied {
        entity: PublicEntity,
        player: PlayerId,
        position: usize,
        source: Option<PublicEntity>,
    },
    TurnEnded {
        player: PlayerId,
        turn: u32,
    },
    Conceded {
        player: PlayerId,
    },
    GameEnded {
        outcome: GameOutcome,
    },
    ChoiceRequested {
        player: PlayerId,
        source: Option<PublicEntity>,
        options: usize,
    },
    ChoiceMade {
        player: PlayerId,
        source: Option<PublicEntity>,
        /// Only the choosing player receives the private option index.
        index: Option<usize>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicEventRecord {
    /// Position in the authoritative event stream. Gaps mean that one or more
    /// internal-only events were deliberately omitted for this viewer.
    pub sequence: u64,
    pub turn: u32,
    pub event: PublicEvent,
}

impl GameState {
    fn entity_snapshot(&self, id: EntityId) -> Option<PublicEntity> {
        self.entity(id).map(|entity| PublicEntity {
            id,
            card_id: entity.card_id.clone(),
        })
    }

    fn identity_visible_in_zone(&self, viewer: PlayerId, entity: &Entity, zone: Zone) -> bool {
        match zone {
            Zone::Hero | Zone::Board | Zone::Weapon | Zone::HeroPower | Zone::Graveyard => true,
            Zone::Hand | Zone::SetAside => entity.controller == viewer,
            Zone::Secret => entity.controller == viewer || entity.is_public_objective(),
            Zone::Deck | Zone::Removed => false,
        }
    }

    fn visible_entity_snapshot(&self, viewer: PlayerId, id: EntityId) -> Option<PublicEntity> {
        let entity = self.entity(id)?;
        self.identity_visible_in_zone(viewer, entity, entity.zone)
            .then(|| PublicEntity {
                id,
                card_id: entity.card_id.clone(),
            })
    }

    fn project_public_event(&self, viewer: PlayerId, event: &GameEvent) -> Option<PublicEvent> {
        let entity = |id| self.entity_snapshot(id);
        let visible = |id| self.visible_entity_snapshot(viewer, id);
        Some(match event {
            GameEvent::GameStarted => PublicEvent::GameStarted,
            GameEvent::TurnStarted { player, turn } => PublicEvent::TurnStarted {
                player: *player,
                turn: *turn,
            },
            GameEvent::CardDrawn {
                player,
                card,
                source,
            } => PublicEvent::CardDrawn {
                player: *player,
                card: (*player == viewer).then(|| entity(*card)).flatten(),
                source: source.and_then(visible),
            },
            GameEvent::CardBurned {
                player,
                card,
                source,
            } => PublicEvent::CardBurned {
                player: *player,
                card: entity(*card)?,
                source: source.and_then(visible),
            },
            GameEvent::CardCreated {
                source,
                player,
                card,
            } => PublicEvent::CardCreated {
                player: *player,
                card: (*player == viewer).then(|| entity(*card)).flatten(),
                source: visible(*source),
            },
            GameEvent::Fatigue { player, amount } => PublicEvent::Fatigue {
                player: *player,
                amount: *amount,
            },
            GameEvent::CardPlayed { player, card, cost } => PublicEvent::CardPlayed {
                player: *player,
                card: visible(*card)?,
                cost: *cost,
            },
            GameEvent::SpellCast {
                player,
                spell,
                generated_by,
                target,
                cost,
                ..
            } => PublicEvent::SpellCast {
                player: *player,
                spell: visible(*spell)?,
                generated_by: generated_by.and_then(visible),
                target: target.and_then(visible),
                cost: *cost,
            },
            GameEvent::SpellTargeted {
                player,
                spell,
                target,
                generated_by,
            } => PublicEvent::SpellTargeted {
                player: *player,
                spell: visible(*spell)?,
                target: visible(*target)?,
                generated_by: generated_by.and_then(visible),
            },
            GameEvent::MinionPlayed { player, minion } => PublicEvent::MinionPlayed {
                player: *player,
                minion: entity(*minion)?,
            },
            GameEvent::WeaponPlayed { player, weapon } => PublicEvent::WeaponPlayed {
                player: *player,
                weapon: entity(*weapon)?,
            },
            GameEvent::LocationPlayed { player, location } => PublicEvent::LocationPlayed {
                player: *player,
                location: entity(*location)?,
            },
            GameEvent::CardCountered { player, card } => PublicEvent::CardCountered {
                player: *player,
                card: entity(*card)?,
            },
            GameEvent::CardDiscarded {
                source,
                player,
                card,
            } => PublicEvent::CardDiscarded {
                player: *player,
                card: entity(*card)?,
                source: visible(*source),
            },
            GameEvent::CardTraded { player, card } => PublicEvent::CardTraded {
                player: *player,
                card: (*player == viewer).then(|| entity(*card)).flatten(),
            },
            GameEvent::TradeDraw { player, .. } => PublicEvent::TradeDraw { player: *player },
            GameEvent::MinionSummoned { player, entity: id } => PublicEvent::MinionSummoned {
                player: *player,
                entity: entity(*id)?,
            },
            GameEvent::Magnetized {
                player,
                attachment,
                target,
            } => PublicEvent::Magnetized {
                player: *player,
                attachment: entity(*attachment)?,
                target: entity(*target)?,
            },
            GameEvent::WeaponEquipped { player, weapon } => PublicEvent::WeaponEquipped {
                player: *player,
                weapon: entity(*weapon)?,
            },
            GameEvent::WeaponDestroyed { player, weapon } => PublicEvent::WeaponDestroyed {
                player: *player,
                weapon: entity(*weapon)?,
            },
            GameEvent::LocationUsed {
                player,
                location,
                target,
            } => PublicEvent::LocationUsed {
                player: *player,
                location: entity(*location)?,
                target: target.and_then(visible),
            },
            GameEvent::LocationDestroyed { player, location } => PublicEvent::LocationDestroyed {
                player: *player,
                location: entity(*location)?,
            },
            GameEvent::HeroPowerUsed {
                player,
                hero_power,
                target,
            } => PublicEvent::HeroPowerUsed {
                player: *player,
                hero_power: entity(*hero_power)?,
                target: target.and_then(visible),
            },
            GameEvent::HeroPowerReplaced {
                source,
                player,
                old,
                new,
            } => PublicEvent::HeroPowerReplaced {
                player: *player,
                source: entity(*source)?,
                old: entity(*old)?,
                new: entity(*new)?,
            },
            GameEvent::HeroReplaced { player, old, new } => PublicEvent::HeroReplaced {
                player: *player,
                old: entity(*old)?,
                new: entity(*new)?,
            },
            GameEvent::SecretPlayed { player, secret } => PublicEvent::SecretPlayed {
                player: *player,
                secret: visible(*secret),
            },
            GameEvent::SecretRevealed { player, secret } => PublicEvent::SecretRevealed {
                player: *player,
                secret: entity(*secret)?,
            },
            GameEvent::ZoneChanged {
                entity: id,
                from,
                to,
            } => {
                let state = self.entity(*id)?;
                let observable = self.identity_visible_in_zone(viewer, state, *from)
                    || self.identity_visible_in_zone(viewer, state, *to);
                if !observable {
                    return None;
                }
                PublicEvent::ZoneChanged {
                    entity: entity(*id)?,
                    from: *from,
                    to: *to,
                }
            }
            GameEvent::ControllerChanged {
                source,
                entity: id,
                from,
                to,
            } => PublicEvent::ControllerChanged {
                source: visible(*source),
                entity: entity(*id)?,
                from: *from,
                to: *to,
            },
            GameEvent::Transformed {
                source,
                entity: id,
                from_card,
                to_card,
            } => PublicEvent::Transformed {
                source: visible(*source),
                entity: visible(*id)?,
                from_card: from_card.clone(),
                to_card: to_card.clone(),
            },
            GameEvent::Attack {
                attacker,
                defender,
                collateral,
            } => PublicEvent::Attack {
                attacker: entity(*attacker)?,
                defender: entity(*defender)?,
                collateral: collateral
                    .iter()
                    .filter_map(|(id, amount)| entity(*id).map(|entity| (entity, *amount)))
                    .collect(),
            },
            GameEvent::Damaged {
                source,
                target,
                amount,
            } => PublicEvent::Damaged {
                source: visible(*source),
                target: entity(*target)?,
                amount: *amount,
            },
            GameEvent::DamagePrevented {
                source,
                target,
                reason,
            } => PublicEvent::DamagePrevented {
                source: visible(*source),
                target: entity(*target)?,
                reason: reason.clone(),
            },
            GameEvent::Healed {
                source,
                target,
                amount,
            } => PublicEvent::Healed {
                source: visible(*source),
                target: entity(*target)?,
                amount: *amount,
            },
            GameEvent::ArmorGained {
                source,
                target,
                amount,
            } => PublicEvent::ArmorGained {
                source: visible(*source),
                target: entity(*target)?,
                amount: *amount,
            },
            GameEvent::OverloadQueued {
                source,
                player,
                amount,
            } => PublicEvent::OverloadQueued {
                player: *player,
                source: visible(*source),
                amount: *amount,
            },
            GameEvent::ManaLocked { player, amount } => PublicEvent::ManaLocked {
                player: *player,
                amount: *amount,
            },
            GameEvent::ManaUnlocked {
                source,
                player,
                amount,
            } => PublicEvent::ManaUnlocked {
                player: *player,
                source: visible(*source),
                amount: *amount,
            },
            GameEvent::OverloadCleared {
                source,
                player,
                pending,
                locked,
            } => PublicEvent::OverloadCleared {
                player: *player,
                source: visible(*source),
                pending: *pending,
                locked: *locked,
            },
            GameEvent::TemporaryManaGained {
                source,
                player,
                amount,
            } => PublicEvent::TemporaryManaGained {
                player: *player,
                source: visible(*source),
                amount: *amount,
            },
            GameEvent::TemporaryManaExpired { player, amount } => {
                PublicEvent::TemporaryManaExpired {
                    player: *player,
                    amount: *amount,
                }
            }
            GameEvent::ManaCrystalsGained {
                source,
                player,
                amount,
                filled,
            } => PublicEvent::ManaCrystalsGained {
                player: *player,
                source: visible(*source),
                amount: *amount,
                filled: *filled,
            },
            GameEvent::ManaCrystalsDestroyed {
                source,
                player,
                amount,
            } => PublicEvent::ManaCrystalsDestroyed {
                player: *player,
                source: visible(*source),
                amount: *amount,
            },
            GameEvent::ManaSpent {
                player,
                source,
                amount,
                temporary,
            } => PublicEvent::ManaSpent {
                player: *player,
                source: visible(*source),
                amount: *amount,
                temporary: *temporary,
            },
            GameEvent::KeywordDisabled {
                source,
                target,
                keyword,
            } => PublicEvent::KeywordDisabled {
                source: visible(*source),
                target: entity(*target)?,
                keyword: keyword.clone(),
            },
            GameEvent::Frozen { source, target } => PublicEvent::Frozen {
                source: visible(*source),
                target: entity(*target)?,
            },
            GameEvent::EntityDied {
                entity: id,
                player,
                position,
                source,
                ..
            } => PublicEvent::EntityDied {
                entity: entity(*id)?,
                player: *player,
                position: *position,
                source: source.and_then(visible),
            },
            GameEvent::TurnEnded { player, turn } => PublicEvent::TurnEnded {
                player: *player,
                turn: *turn,
            },
            GameEvent::Conceded { player } => PublicEvent::Conceded { player: *player },
            GameEvent::GameEnded { outcome } => PublicEvent::GameEnded { outcome: *outcome },
            GameEvent::ChoiceRequested {
                player,
                source,
                options,
            } => PublicEvent::ChoiceRequested {
                player: *player,
                source: visible(*source),
                options: *options,
            },
            GameEvent::ChoiceMade {
                player,
                source,
                index,
            } => PublicEvent::ChoiceMade {
                player: *player,
                source: visible(*source),
                index: (*player == viewer).then_some(*index),
            },
            GameEvent::PlayerScriptDataChanged { .. }
            | GameEvent::RandomChoiceMade { .. }
            | GameEvent::RandomCardsSampled { .. }
            | GameEvent::RandomEntitiesSampled { .. } => return None,
        })
    }

    pub(crate) fn record_event(&mut self, event: GameEvent) {
        let sequence = self.log.len() as u64;
        for viewer in [PlayerId::ONE, PlayerId::TWO] {
            if let Some(event) = self.project_public_event(viewer, &event) {
                Arc::make_mut(&mut self.public_logs[viewer.index()]).push(PublicEventRecord {
                    sequence,
                    turn: self.turn,
                    event,
                });
            }
        }
        self.log.push(event);
    }

    pub fn public_history(&self, viewer: PlayerId) -> &[PublicEventRecord] {
        &self.public_logs[viewer.index()]
    }
}
