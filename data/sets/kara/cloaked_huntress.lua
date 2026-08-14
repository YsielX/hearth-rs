local function is_secret(definition)
    for _, keyword in ipairs(definition.keywords or {}) do
        if keyword == "secret" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "KAR_006",
    name = "Cloaked Huntress",
    text = "Your <b>Secrets</b> cost (0).",
    set = "KARA",
    type = "minion",
    class = "hunter",
    rarity = "common",
    cost = 3,
    attack = 3,
    health = 4,
    auras = {{
        active_zones = { "board" },
        cost_set = 0,
        targets = function(ctx, self)
            local result = {}
            for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
                if is_secret(ctx:card_definition(ctx:entity(entity).card_id)) then
                    result[#result + 1] = entity
                end
            end
            return result
        end,
    }},
}
