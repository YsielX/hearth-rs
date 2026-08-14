return {
    api_version = 1,
    id = "TRL_015",
    name = "Ticket Scalper",
    text = "<b>Overkill</b>: Draw 2 cards.",
    set = "TROLL",
    type = "minion",
    class = "neutral",
    cost = 4,
    attack = 5,
    health = 3,
    tags = { "pirate" },
    keywords = { "overkill" },
    on_overkill = function(ctx, self, target)
        ctx:draw(ctx:controller(self), 2)
    end,
}
