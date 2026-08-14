local function dragon(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "dragon" or tag == "all" then return true end
    end
    return false
end
return {
    api_version = 1, id = "CFM_662", name = "Dragonfire Potion",
    text = "[x]Deal $5 damage to all\nminions except Dragons.",
    set = "GANGS", type = "spell", class = "priest", rarity = "epic", spell_school = "fire", cost = 5,
    on_play = function(ctx, self)
        local targets = {}
        for _, entity in ipairs(ctx:all_characters()) do
            if ctx:entity(entity).type == "minion" and not dragon(ctx, entity) then targets[#targets + 1] = entity end
        end
        ctx:damage_all(targets, 5)
    end,
}
