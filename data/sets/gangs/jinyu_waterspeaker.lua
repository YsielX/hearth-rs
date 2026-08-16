return {
    api_version = 1, id = "CFM_061", name = "Jinyu Waterspeaker",
    text = "[x]<b>Battlecry:</b> Restore #6 Health.\n<b>Overload:</b> (1)", set = "GANGS",
    type = "minion", class = "shaman", rarity = "rare", cost = 4, attack = 4,
    health = 6, keywords = { "battlecry", "overload" },
    keyword_params = { overload = 1 }, target_mode = "required_if_available",
    targets = function(ctx) return ctx:characters() end,
    on_battlecry = function(ctx, self, target) if target then cardlib.effects.heal(ctx, target, 6) end end,
}
