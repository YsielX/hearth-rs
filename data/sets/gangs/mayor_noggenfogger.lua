return {
    api_version = 1, id = "CFM_670", name = "Mayor Noggenfogger",
    text = "All targets are chosen randomly.", set = "GANGS", type = "minion",
    rarity = "legendary", cost = 9, attack = 5, health = 4,
    auras = {{
        keywords = { "randomize_targets" },
        targets = function(ctx, self)
            for _, keyword in ipairs(ctx:entity(self).keywords or {}) do
                if keyword == "dormant" then return {} end
            end
            local player = ctx:controller(self)
            return {
                ctx:player(player).hero,
                ctx:player(ctx:opponent(player)).hero,
            }
        end,
    }},
}
