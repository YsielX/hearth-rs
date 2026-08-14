return {
    api_version = 1,
    id = "BRM_008",
    name = "Dark Iron Skulker",
    text = "<b>Battlecry:</b> Deal 2 damage to all undamaged\nenemy minions.",
    set = "BRM",
    type = "minion",
    class = "rogue",
    rarity = "rare",
    cost = 4,
    attack = 4,
    health = 3,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local targets = {}
        for _, entity in ipairs(ctx:enemy_characters(self)) do
            local minion = ctx:entity(entity)
            if minion.type == "minion" and minion.damage == 0 then
                targets[#targets + 1] = entity
            end
        end
        if #targets > 0 then ctx:damage_all(targets, 2) end
    end,
}
