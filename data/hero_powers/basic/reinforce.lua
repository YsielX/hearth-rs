return {
    api_version = 1,
    module_type = "hero_power",
    id = "HERO_04bp",
    name = "Reinforce",
    text = "<b>Hero Power</b>\nSummon a {0} Silver Hand Recruit.",
    set = "LEGACY",
    class = "paladin",
    cost = 2,
    on_play = function(ctx, self)
        ctx:summon(ctx:controller(self), "CS2_101t")
    end,
    tokens = {
        {
            id = "CS2_101t", name = "Silver Hand Recruit", text = "",
            set = "LEGACY", type = "minion", class = "paladin",
            cost = 1, attack = 1, health = 1,
        },
    },
}
