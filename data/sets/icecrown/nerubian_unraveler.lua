return {
    api_version = 1,
    id = "ICC_706",
    name = "Nerubian Unraveler",
    text = "Spells cost (2) more.",
    set = "ICECROWN",
    type = "minion",
    rarity = "epic",
    cost = 6,
    attack = 5,
    health = 5,
    auras = {
        {
            active_zones = { "board" },
            cost = 2,
            targets = function(ctx, self)
                local owner = ctx:controller(self)
                local result = {}
                for _, player in ipairs({ owner, ctx:opponent(owner) }) do
                    for _, zone in ipairs({ ctx:hand(player), ctx:deck(player) }) do
                        for _, entity in ipairs(zone) do
                            if ctx:entity(entity).type == "spell" then result[#result + 1] = entity end
                        end
                    end
                end
                return result
            end,
        },
    },
}
