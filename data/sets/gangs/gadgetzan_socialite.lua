return {
    api_version = 1, id = "CFM_659", name = "Gadgetzan Socialite",
    text = "<b>Battlecry:</b> Restore #2 Health.", set = "GANGS", type = "minion", rarity = "common",
    cost = 2, attack = 2, health = 2, keywords = { "battlecry" }, target_mode = "required",
    targets = function(ctx, self) return ctx:all_characters() end,
    on_battlecry = function(ctx, self, target) ctx:heal(target, 2) end,
}
