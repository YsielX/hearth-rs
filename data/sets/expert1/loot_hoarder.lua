return {
    api_version = 1,
    id = "EX1_096",
    name = "Loot Hoarder",
    text = "<b>Deathrattle:</b> Draw a card.",
    set = "EXPERT1",
    type = "minion",
    cost = 2,
    attack = 2,
    health = 1,
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        ctx:draw(ctx:controller(self), 1)
    end,
}
