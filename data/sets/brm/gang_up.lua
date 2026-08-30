return {
    api_version = 1,
    id = "BRM_007",
    name = "Gang Up",
    text = "Choose a minion. Shuffle 3 copies of it into your deck.",
    set = "BRM",
    type = "spell",
    class = "rogue",
    rarity = "common",
    cost = 2,
    target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
    on_play = function(ctx, self, target)
        local player = ctx:controller(self)
        local card_id = ctx:entity(target).card_id
        for _ = 1, 3 do cardlib.effects.shuffle_card_into_deck(ctx, player, card_id) end
    end,
}
