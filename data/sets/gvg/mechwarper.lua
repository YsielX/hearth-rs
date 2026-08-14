return {
    api_version = 1, id = "GVG_006", name = "Mechwarper", text = "Your Mechs cost (1) less.",
    set = "GVG", type = "minion", rarity = "common", cost = 4, attack = 4, health = 4,
    tags = { "mech" },
    auras = {{
        cost = -1,
        targets = function(ctx, self)
            local targets = {}
            local player = ctx:controller(self)
            for _, zone in ipairs({ ctx:hand(player), ctx:deck(player) }) do
                for _, entity in ipairs(zone) do
                    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags) do
                        if tag == "mech" then targets[#targets + 1] = entity break end
                    end
                end
            end
            return targets
        end,
    }},
}
