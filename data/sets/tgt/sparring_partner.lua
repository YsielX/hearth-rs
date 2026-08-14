return {
    api_version = 1, id = "AT_069", name = "Sparring Partner",
    text = "<b>Taunt</b>\n<b>Battlecry:</b> Give a\nminion <b>Taunt</b>.", set = "TGT", type = "minion",
    class = "warrior", rarity = "rare", cost = 2, attack = 3, health = 2,
    keywords = { "taunt", "battlecry" }, target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
    on_battlecry = function(ctx, self, target) ctx:grant_keyword(target, "taunt") end,
}
