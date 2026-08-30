local function summon_two(ctx, self)
    ctx:summon_copy(ctx:controller(self), self)
    ctx:summon_copy(ctx:controller(self), self)
end

return {
    api_version = 1, id = "CFM_668", name = "Doppelgangster",
    text = "<b>Battlecry:</b> Summon 2 copies of this minion.",
    set = "GANGS", type = "minion", rarity = "rare", cost = 5, attack = 2, health = 2,
    keywords = { "battlecry" },
    on_battlecry = summon_two,
    tokens = {
        {
            id = "CFM_668t", rarity = "rare", name = "Doppelgangster",
            text = "<b>Battlecry:</b> Summon 2 copies of this minion.",
            set = "GANGS", type = "minion", cost = 5, attack = 2, health = 2,
            keywords = { "battlecry" }, on_battlecry = summon_two,
        },
        {
            id = "CFM_668t2", rarity = "rare", name = "Doppelgangster",
            text = "<b>Battlecry:</b> Summon 2 copies of this minion.",
            set = "GANGS", type = "minion", cost = 5, attack = 2, health = 2,
            keywords = { "battlecry" }, on_battlecry = summon_two,
        },
    },
}
