local function is_secret(definition)
    for _, keyword in ipairs(definition.keywords or {}) do
        if keyword == "secret" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "CFM_760",
    name = "Kabal Crystal Runner",
    text = "Costs (2) less for each <b>Secret</b> you've played this game.",
    set = "GANGS",
    type = "minion",
    class = "mage",
    rarity = "rare",
    cost = 6,
    attack = 6,
    health = 5,
    tags = { "draenei" },
    auras = {{
        active_zones = { "hand" },
        targets = function(ctx, self) return { self } end,
        cost = function(ctx, self)
            local count = 0
            for _, card_id in ipairs(ctx:cards_played(ctx:controller(self))) do
                if is_secret(ctx:card_definition(card_id)) then count = count + 1 end
            end
            return -2 * count
        end,
    }},
}
