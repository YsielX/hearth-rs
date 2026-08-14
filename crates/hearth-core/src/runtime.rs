use crate::{
    AuraSpec, CardActionSpec, CardDefinition, ChoiceValue, EffectSpec, EntityId, GameState,
    ScriptEvent,
};

/// The stable boundary between the deterministic Rust engine and a card script host.
pub trait CardRuntime {
    fn pack_hash(&self) -> &str;

    fn definition(&self, card_id: &str) -> Option<&CardDefinition>;

    fn card_ids(&self) -> Vec<String>;

    fn keyword_i32_rule(
        &self,
        state: &GameState,
        entity: EntityId,
        rule: &str,
        initial: i32,
        other: Option<EntityId>,
    ) -> Result<i32, String>;

    fn keyword_bool_rule(
        &self,
        state: &GameState,
        entity: EntityId,
        rule: &str,
        initial: bool,
        other: Option<EntityId>,
    ) -> Result<bool, String>;

    fn valid_targets(&self, state: &GameState, source: EntityId) -> Result<Vec<EntityId>, String>;

    fn location_targets(
        &self,
        state: &GameState,
        source: EntityId,
    ) -> Result<Vec<EntityId>, String>;

    fn card_actions(
        &self,
        state: &GameState,
        source: EntityId,
    ) -> Result<Vec<CardActionSpec>, String>;

    fn action_targets(
        &self,
        state: &GameState,
        source: EntityId,
        action: &str,
    ) -> Result<Vec<EntityId>, String>;

    fn on_card_action(
        &self,
        state: &GameState,
        source: EntityId,
        action: &str,
        spent: u8,
        target: Option<EntityId>,
    ) -> Result<Vec<EffectSpec>, String>;

    fn on_play(
        &self,
        state: &GameState,
        source: EntityId,
        target: Option<EntityId>,
    ) -> Result<Vec<EffectSpec>, String>;

    fn on_location_use(
        &self,
        state: &GameState,
        source: EntityId,
        target: Option<EntityId>,
    ) -> Result<Vec<EffectSpec>, String>;

    fn on_event(
        &self,
        state: &GameState,
        listener: EntityId,
        event: &ScriptEvent,
    ) -> Result<Vec<EffectSpec>, String>;

    fn on_resume(
        &self,
        state: &GameState,
        source: EntityId,
        continuation_owner: Option<&str>,
        hook: &str,
        choice: &ChoiceValue,
    ) -> Result<Vec<EffectSpec>, String>;

    fn on_continue(
        &self,
        state: &GameState,
        source: EntityId,
        continuation_owner: Option<&str>,
        hook: &str,
        payload: Option<&ChoiceValue>,
    ) -> Result<Vec<EffectSpec>, String>;

    fn auras(&self, state: &GameState, source: EntityId) -> Result<Vec<AuraSpec>, String>;
}
