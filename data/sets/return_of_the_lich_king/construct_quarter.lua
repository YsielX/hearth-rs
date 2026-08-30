local card = {
    api_version = 1,
    id = "NX2_036",
    name = "Construct Quarter",
    text = "[x]Destroy a friendly\nminion to summon a\n4/5 Undead with <b>Rush</b>.",
    set = "RETURN_OF_THE_LICH_KING",
    type = "location",
    class = "death_knight",
    rarity = "rare",
    cost = 3,
    health = 3,
    target_mode = "required",
    location_targets = function(ctx, self) return ctx:friendly_minions(self) end,
}

function card.on_location_use(ctx, self, target)
    cardlib.effects.destroy(ctx, target)
    cardlib.effects.summon_with_stats(ctx, ctx:controller(self), "NX2_036t", 4, 5)
end

card.tokens = {{
    id = "NX2_036t",
    name = "Shambling Horror",
    text = "<b>Rush</b>",
    set = "RETURN_OF_THE_LICH_KING",
    type = "minion",
    class = "death_knight",
    collectible = false,
    cost = 5,
    attack = 5,
    health = 5,
    tags = { "undead" },
    keywords = { "rush" },
}}

return card
