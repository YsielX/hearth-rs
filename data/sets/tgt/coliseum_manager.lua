return {
    api_version = 1,
    id = "AT_110",
    name = "Coliseum Manager",
    text = "<b>Inspire:</b> Return this minion to your hand.",
    set = "TGT",
    type = "minion",
    rarity = "rare",
    cost = 3,
    attack = 2,
    health = 5,
    keywords = { "inspire" },
    on_inspire = function(ctx, self) ctx:move(self, "hand") end,
}
