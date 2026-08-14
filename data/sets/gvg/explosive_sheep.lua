return {
    api_version = 1, id = "GVG_076", name = "Explosive Sheep",
    text = "<b>Deathrattle:</b> Deal 2 damage to all minions.", set = "GVG", type = "minion",
    rarity = "common", cost = 2, attack = 1, health = 1, tags = { "mech", "beast" },
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self) ctx:damage_all(ctx:minions(), 2) end,
}
