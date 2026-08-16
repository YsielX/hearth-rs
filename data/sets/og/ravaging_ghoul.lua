return {
    api_version = 1, id = "OG_149", name = "Ravaging Ghoul",
    text = "<b>Battlecry:</b> Deal 1 damage to all other minions.", set = "OG", type = "minion",
    class = "warrior", rarity = "common", cost = 3, attack = 3, health = 3,
    tags = { "undead" }, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local targets = {}
        for _, minion in ipairs(ctx:minions()) do if minion ~= self then targets[#targets + 1] = minion end end
        cardlib.effects.damage_all(ctx, targets, 1)
    end,
}
