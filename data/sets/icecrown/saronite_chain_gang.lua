return {
    api_version = 1,
    id = "ICC_466",
    name = "Saronite Chain Gang",
    text = "[x]<b>Taunt</b>\n<b>Battlecry:</b> Summon a\nSaronite Chain Gang.",
    set = "ICECROWN",
    type = "minion",
    rarity = "rare",
    cost = 4,
    attack = 2,
    health = 3,
    tags = { "draenei" },
    keywords = { "taunt", "battlecry" },
    on_battlecry = function(ctx, self)
        ctx:summon(ctx:controller(self), "ICC_466")
    end,
}
