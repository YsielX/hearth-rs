return {
    api_version = 1,
    id = "REV_956", rarity = "common",
    name = "Priest of the Deceased",
    text = "<b>Taunt</b>\n<b>Infuse (3):</b> Gain +2/+2.",
    set = "REVENDRETH",
    type = "minion",
    cost = 2,
    attack = 2,
    health = 3,
    keywords = { "taunt", "infuse" },
    keyword_params = { infuse = 3 },
    on_infuse = function(ctx, self)
        cardlib.effects.buff(ctx, self, 2, 2)
    end,
}
