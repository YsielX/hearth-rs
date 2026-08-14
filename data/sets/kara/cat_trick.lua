local card = {
    api_version = 1,
    id = "KAR_004",
    name = "Cat Trick",
    text = "<b>Secret:</b> After your opponent casts a spell, summon a 4/2 Panther with <b>Stealth</b>.",
    set = "KARA",
    type = "spell",
    class = "hunter",
    rarity = "rare",
    cost = 2,
    keywords = { "secret" },
}

card.triggers = {{
    event = "spell_cast",
    timing = "after",
    active_zones = { "secret" },
    condition = function(ctx, self, event)
        local player = ctx:controller(self)
        return event.player == ctx:opponent(player) and event.player_cast and #ctx:board(player) < 7
    end,
    effect = function(ctx, self)
        ctx:reveal_secret(self)
        ctx:summon(ctx:controller(self), "KAR_004a")
    end,
}}

card.tokens = {{
    id = "KAR_004a",
    name = "Cat in a Hat",
    text = "<b>Stealth</b>",
    set = "KARA",
    type = "minion",
    class = "hunter",
    cost = 3,
    attack = 4,
    health = 2,
    tags = { "beast" },
    keywords = { "stealth" },
}}

return card
