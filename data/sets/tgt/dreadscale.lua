local function is_dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "dormant" then return true end
    end
    return false
end

return {
    api_version = 1, id = "AT_063t", name = "Dreadscale",
    text = "At the end of your turn, deal 1 damage to all enemies.",
    set = "TGT", type = "minion", class = "hunter", rarity = "legendary",
    collectible = true, cost = 3, attack = 4, health = 2, tags = { "beast" },
    triggers = {
        {
            event = "turn_ended", timing = "after", active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self)
                local targets = {}
                for _, enemy in ipairs(ctx:enemy_characters(self)) do
                    if not is_dormant(ctx, enemy) then targets[#targets + 1] = enemy end
                end
                if #targets > 0 then cardlib.effects.damage_all(ctx, targets, 1) end
            end,
        },
    },
}
