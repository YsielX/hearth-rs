use std::collections::BTreeMap;

use hearth_core::{
    CardKind, EntityId, EntityView, LegalAction, PlayerCommand, PlayerId, PlayerStateView,
    PlayerView, Zone,
};

pub(super) fn entity(
    id: u64,
    kind: CardKind,
    controller: PlayerId,
    attack: i32,
    health: i32,
    cost: u8,
) -> EntityView {
    EntityView {
        id: EntityId(id),
        card_id: format!("CARD_{id}"),
        kind,
        owner: controller,
        controller,
        zone: match kind {
            CardKind::Hero => Zone::Hero,
            CardKind::HeroPower => Zone::HeroPower,
            _ => Zone::Board,
        },
        attack,
        max_health: health,
        damage: 0,
        armor: 0,
        cost,
        spell_damage: 0,
        exhausted: false,
        frozen: false,
        attacks_this_turn: 0,
        location_cooldown: 0,
        keywords: Vec::new(),
        silenced: false,
        public_cards: Vec::new(),
        public_counters: Default::default(),
    }
}

pub(super) fn view(mana: u8, extra: Vec<EntityView>) -> PlayerView {
    let mut entities = BTreeMap::new();
    for entity in [
        entity(1, CardKind::Hero, PlayerId::ONE, 0, 30, 0),
        entity(2, CardKind::Hero, PlayerId::TWO, 0, 30, 0),
        entity(3, CardKind::HeroPower, PlayerId::ONE, 0, 1, 2),
        entity(4, CardKind::HeroPower, PlayerId::TWO, 0, 1, 2),
    ]
    .into_iter()
    .chain(extra)
    {
        entities.insert(entity.id, entity);
    }
    let player = |id, hero, power| PlayerStateView {
        id,
        class: "neutral".to_owned(),
        hero,
        deck_size: 20,
        hand_size: 0,
        hand: Vec::new(),
        sideboards: BTreeMap::new(),
        board: entities
            .values()
            .filter(|entity| entity.controller == id && entity.kind == CardKind::Minion)
            .map(|entity| entity.id)
            .collect(),
        weapon: None,
        hero_power: power,
        hero_power_used: false,
        hero_power_uses_this_turn: 0,
        secrets_count: 0,
        secrets: Vec::new(),
        public_objectives: Vec::new(),
        mana: if id == PlayerId::ONE { mana } else { 0 },
        max_mana: mana,
        temporary_mana: 0,
        resources: BTreeMap::new(),
        resources_spent: BTreeMap::new(),
        public_statuses: Vec::new(),
        public_counters: Default::default(),
        overload_pending: 0,
        overloaded_mana: 0,
        fatigue: 0,
        cards_played_this_turn: 0,
    };
    PlayerView {
        viewer: PlayerId::ONE,
        turn: 1,
        active_player: PlayerId::ONE,
        input_player: PlayerId::ONE,
        players: [
            player(PlayerId::ONE, EntityId(1), EntityId(3)),
            player(PlayerId::TWO, EntityId(2), EntityId(4)),
        ],
        entities,
        outcome: None,
        mulligan_eligible: Vec::new(),
        pending_input: None,
        history: Default::default(),
    }
}

pub(super) fn legal(command: PlayerCommand, mana_cost: u8) -> LegalAction {
    LegalAction {
        command,
        mana_cost,
        semantic_card_id: None,
    }
}

pub(super) fn spell(id: &str, text: &str) -> hearth_core::CardDefinition {
    serde_json::from_value(serde_json::json!({
        "id": id, "name": id, "text": text, "kind": "spell", "cost": 1,
        "attack": 0, "health": 0, "keywords": []
    }))
    .unwrap()
}
