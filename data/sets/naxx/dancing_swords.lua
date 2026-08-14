return {
    api_version = 1,
    id = "FP1_029",
    name = "Dancing Swords",
    text = "<b>Deathrattle:</b> Your opponent draws a card.",
    set = "NAXX",
    type = "minion",
    rarity = "common",
    cost = 3,
    attack = 4,
    health = 4,
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        ctx:draw(ctx:opponent(ctx:controller(self)), 1)
    end,
}
