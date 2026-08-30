return {
    api_version = 1, id = "UNG_022", name = "Mirage Caller",
    text = "<b>Battlecry:</b> Choose a minion. Summon a 1/1 copy of it.",
    set = "UNGORO", type = "minion", class = "priest", rarity = "rare",
    cost = 3, attack = 2, health = 3, keywords = { "battlecry" }, target_mode = "required_if_available",
    targets = function(ctx, self) return ctx:minions() end,
    on_battlecry = function(ctx, self, target)
        if target then cardlib.effects.summon_copy_with_stats(ctx, ctx:controller(self), target, 1, 1) end
    end,
}
