local function dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do if keyword == "dormant" then return true end end
    return false
end

return {
    api_version = 1, id = "UNG_015", name = "Sunkeeper Tarim",
    text = "<b>Taunt</b>\n<b>Battlecry:</b> Set all other minions' Attack and Health to 3.",
    set = "UNGORO", type = "minion", class = "paladin", rarity = "legendary",
    cost = 6, attack = 3, health = 7, keywords = { "taunt", "battlecry" },
    on_battlecry = function(ctx, self)
        local targets = {}
        for _, entity in ipairs(ctx:minions()) do
            if entity ~= self and not dormant(ctx, entity) then targets[#targets + 1] = entity end
        end
        ctx:modify_all(targets, {
            attack = 3, health = 3, operation = "final_set", silenciable = true, reset_damage = true,
        })
    end,
}
