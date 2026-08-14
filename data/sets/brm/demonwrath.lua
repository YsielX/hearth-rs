local function is_demon(ctx, entity)
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags) do
        if tag == "demon" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "BRM_005",
    name = "Demonwrath",
    text = "[x]Deal $2 damage to all\nminions except Demons.",
    set = "BRM",
    type = "spell",
    class = "warlock",
    rarity = "rare",
    spell_school = "fel",
    cost = 3,
    on_play = function(ctx, self)
        local targets = {}
        for _, minion in ipairs(ctx:minions()) do
            if not is_demon(ctx, minion) then targets[#targets + 1] = minion end
        end
        if #targets > 0 then ctx:damage_all(targets, 2) end
    end,
}
