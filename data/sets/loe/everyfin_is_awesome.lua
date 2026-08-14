local function friendly_murlocs(ctx, self)
    local count = 0
    for _, entity in ipairs(ctx:board(ctx:controller(self))) do
        local definition = ctx:card_definition(ctx:entity(entity).card_id)
        for _, tag in ipairs(definition.tags or {}) do
            if tag == "murloc" or tag == "all" then
                count = count + 1
                break
            end
        end
    end
    return count
end

return {
    api_version = 1,
    id = "LOE_113",
    name = "Everyfin is Awesome",
    text = "Give your minions +2/+2.\nCosts (1) less for each Murloc you control.",
    set = "LOE",
    type = "spell",
    class = "shaman",
    rarity = "rare",
    cost = 7,
    auras = {
        {
            active_zones = { "hand", "deck" },
            cost = function(ctx, self) return -friendly_murlocs(ctx, self) end,
            targets = function(ctx, self) return { self } end,
        },
    },
    on_play = function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            ctx:buff(minion, 2, 2)
        end
    end,
}
