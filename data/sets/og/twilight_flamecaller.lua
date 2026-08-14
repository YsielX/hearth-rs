return {
    api_version = 1, id = "OG_083", name = "Twilight Flamecaller",
    text = "<b>Battlecry:</b> Deal 1 damage to all enemy minions.",
    set = "OG", type = "minion", class = "mage", rarity = "common",
    cost = 3, attack = 2, health = 2, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local targets = {}
        for _, minion in ipairs(ctx:enemy_characters(self)) do
            if ctx:entity(minion).type == "minion" then targets[#targets + 1] = minion end
        end
        if #targets > 0 then ctx:damage_all(targets, 1) end
    end,
}
