local function friendly_totems(ctx, self)
    local count = 0
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        local entity = ctx:entity(minion)
        local dormant = false
        for _, keyword in ipairs(entity.keywords) do
            if keyword == "dormant" then dormant = true break end
        end
        if not dormant then
            for _, tag in ipairs(ctx:card_definition(entity.card_id).tags or {}) do
                if tag == "totem" or tag == "all" then count = count + 1 break end
            end
        end
    end
    return count
end

return {
    api_version = 1, id = "OG_023", name = "Primal Fusion",
    text = "Give a minion +1/+1 for each of your Totems.", set = "OG", type = "spell",
    class = "shaman", rarity = "common", cost = 1, spell_school = "nature",
    target_mode = "required", targets = function(ctx) return ctx:minions() end,
    on_play = function(ctx, self, target)
        local amount = friendly_totems(ctx, self)
        if amount > 0 then cardlib.effects.buff(ctx, target, amount, amount) end
    end,
}
