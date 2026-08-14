return {
    api_version = 1, id = "AT_001", name = "Flame Lance",
    text = "Deal $25 damage\nto a minion.", set = "TGT", type = "spell",
    class = "mage", rarity = "common", spell_school = "fire", cost = 5,
    target_mode = "required", targets = function(ctx) return ctx:minions() end,
    on_play = function(ctx, self, target) ctx:damage(target, 25) end,
}
