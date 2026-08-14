return {
    api_version = 1, id = "GVG_013", name = "Cogmaster",
    text = "Has +2 Attack while you have a Mech.", set = "GVG", type = "minion",
    rarity = "common", cost = 1, attack = 1, health = 2,
    auras = {{
        attack = function(ctx, self)
            for _, minion in ipairs(ctx:friendly_minions(self)) do
                for _, tag in ipairs(ctx:card_definition(ctx:entity(minion).card_id).tags) do
                    if tag == "mech" then return 2 end
                end
            end
            return 0
        end,
        targets = function(ctx, self) return { self } end,
    }},
}
