return {
    api_version = 1, id = "CFM_062", name = "Grimestreet Protector",
    text = "[x]<b>Taunt</b>\n<b>Battlecry:</b> Give adjacent\n minions <b>Divine Shield</b>.",
    set = "GANGS", type = "minion", class = "paladin", rarity = "rare",
    cost = 7, attack = 6, health = 6, keywords = { "taunt", "battlecry" },
    on_battlecry = function(ctx, self)
        for _, minion in ipairs(ctx:adjacent_minions(self)) do ctx:grant_keyword(minion, "divine_shield") end
    end,
}
