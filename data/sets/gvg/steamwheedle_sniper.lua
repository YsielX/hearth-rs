return {
    api_version = 1,
    id = "GVG_087",
    name = "Steamwheedle Sniper",
    text = "Your Hero Power can target minions.",
    set = "GVG",
    type = "minion",
    class = "hunter",
    rarity = "epic",
    cost = 2,
    attack = 2,
    health = 3,

    auras = {
        {
            keywords = { "hero_power_can_target_minions" },
            targets = function(ctx, self)
                return { ctx:player(ctx:controller(self)).hero_power }
            end,
        },
    },
}
