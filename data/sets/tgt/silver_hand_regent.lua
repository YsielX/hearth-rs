return {
    api_version = 1,
    id = "AT_100",
    name = "Silver Hand Regent",
    text = "<b>Inspire:</b> Summon a {0} Silver Hand Recruit.",
    set = "TGT",
    type = "minion",
    rarity = "common",
    cost = 3,
    attack = 3,
    health = 3,
    keywords = { "inspire" },
    on_inspire = function(ctx, self) ctx:summon(ctx:controller(self), "CS2_101t") end,
}
