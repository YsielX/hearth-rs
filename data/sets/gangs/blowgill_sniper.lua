return {
    api_version = 1, id = "CFM_647", name = "Blowgill Sniper",
    text = "<b>Battlecry:</b> Deal 1 damage.", set = "GANGS", type = "minion", rarity = "common",
    cost = 2, attack = 2, health = 1, tags = { "murloc" }, keywords = { "battlecry" },
    target_mode = "required", targets = function(ctx, self) return ctx:all_characters() end,
    on_battlecry = function(ctx, self, target) ctx:damage(target, 1) end,
}
