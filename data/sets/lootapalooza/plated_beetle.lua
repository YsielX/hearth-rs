return {
    api_version = 1,
    id = "LOOT_413",
    name = "Plated Beetle",
    text = "<b>Deathrattle:</b> Gain 3 Armor.",
    set = "LOOTAPALOOZA",
    type = "minion",
    rarity = "common",
    cost = 2,
    attack = 2,
    health = 3,
    tags = { "beast" },
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        ctx:gain_armor(ctx:controller(self), 3)
    end,
}
