return {
    api_version = 1,
    id = "EX1_144", spell_school = "shadow", rarity = "common",
    name = "Shadowstep",
    text = "Return a friendly minion to your hand. It costs (2) less.",
    set = "EXPERT1",
    type = "spell",
    class = "rogue",
    cost = 0,
    target_mode = "required",
    targets = function(ctx, self) return ctx:friendly_minions(self) end,
    on_play = function(ctx, self, target)
        ctx:move(target, "hand")
        cardlib.effects.modify(ctx, target, { stat = "cost", operation = "add", value = -2 })
    end,
}
