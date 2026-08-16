local function frozen(ctx, entity) return ctx:entity(entity).frozen end
return {
    api_version = 1, id = "OG_081", name = "Shatter",
    text = "Destroy a <b>Frozen</b> minion.", set = "OG", type = "spell",
    class = "mage", rarity = "common", spell_school = "frost", cost = 2,
    target_mode = "required",
    targets = function(ctx)
        local result = {}
        for _, minion in ipairs(ctx:minions()) do
            if frozen(ctx, minion) then result[#result + 1] = minion end
        end
        return result
    end,
    on_play = function(ctx, self, target) cardlib.effects.destroy(ctx, target) end,
}
