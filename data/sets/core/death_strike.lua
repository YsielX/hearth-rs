return {
    api_version = 1,
    id = "RLK_024",
    name = "Death Strike",
    text = "<b>Lifesteal</b>\nDeal $6 damage\nto a minion.",
    set = "CORE",
    type = "spell",
    class = "death_knight",
    rarity = "common",
    cost = 4,
    keywords = { "lifesteal" },
    rune_cost = { blood = 1 },
    target_mode = "required",
    targets = function(ctx)
        return ctx:minions()
    end,
    on_play = function(ctx, self, target)
        cardlib.effects.damage(ctx, target, 6)
    end,
}
