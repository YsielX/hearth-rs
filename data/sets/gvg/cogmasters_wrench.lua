local function has_friendly_mech(ctx, self)
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        local definition = ctx:card_definition(ctx:entity(minion).card_id)
        for _, tag in ipairs(definition.tags) do
            if tag == "mech" then return true end
        end
    end
    return false
end

return {
    api_version = 1,
    id = "GVG_024",
    name = "Cogmaster's Wrench",
    text = "Has +2 Attack while you have a Mech.",
    set = "GVG",
    type = "weapon",
    class = "rogue",
    rarity = "epic",
    cost = 3,
    attack = 1,
    health = 3,
    auras = {
        {
            active_zones = { "weapon" },
            attack = function(ctx, self)
                if has_friendly_mech(ctx, self) then return 2 end
                return 0
            end,
            targets = function(ctx, self)
                return { self }
            end,
        },
    },
}
