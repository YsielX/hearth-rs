return {
    api_version = 1,
    module_type = "hero_power",
    id = "ICC_834h",
    name = "Bladestorm",
    text = "Deal $1 damage to all minions.",
    set = "ICECROWN",
    class = "warrior",
    cost = 2,
    on_play = function(ctx, self) cardlib.effects.damage_all(ctx, ctx:minions(), 1) end,
}
