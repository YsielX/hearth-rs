use crate::{PlayerCommand, PlayerView};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegalAction {
    pub command: PlayerCommand,
    /// Mana committed by this command in the current authoritative state.
    /// Health-paid cards and non-resource actions report zero.
    pub mana_cost: u8,
}

pub trait PlayerController {
    fn choose_action(
        &mut self,
        view: &PlayerView,
        legal_actions: &[LegalAction],
    ) -> Result<PlayerCommand, String>;
}
