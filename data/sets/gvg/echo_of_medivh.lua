return {
    api_version = 1,
    id = "GVG_005",
    name = "Echo of Medivh",
    text = "Put a copy of each friendly minion into your hand.",
    set = "GVG",
    type = "spell",
    class = "mage",
    spell_school = "arcane",
    rarity = "epic",
    cost = 4,
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            cardlib.effects.give_card(ctx, player, ctx:entity(minion).card_id)
        end
    end,
}
