local function is_demon(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "demon" or tag == "all" then return true end
    end
    return false
end

return {
    api_version = 1, id = "AT_024", name = "Demonfuse", text = "Give a Demon +3/+3.",
    set = "TGT", type = "spell", class = "warlock", rarity = "common", cost = 2,
    spell_school = "shadow", target_mode = "required",
    targets = function(ctx)
        local result = {}
        for _, minion in ipairs(ctx:minions()) do
            if is_demon(ctx, minion) then result[#result + 1] = minion end
        end
        return result
    end,
    on_play = function(ctx, self, target) ctx:buff(target, 3, 3) end,
}
