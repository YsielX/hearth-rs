return {
    api_version = 1,
    id = "EX1_015",
    name = "Novice Engineer",
    text = "<b>Battlecry:</b> Draw a card.",
    set = "LEGACY",
    type = "minion",
    cost = 2,
    attack = 1,
    health = 1,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        ctx:draw(ctx:controller(self), 1)
    end,
}
