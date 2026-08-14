local function deaths_this_turn(ctx, self)
    local player = ctx:controller(self)
    return #ctx:minions_died_this_turn(player)
        + #ctx:minions_died_this_turn(ctx:opponent(player))
end

return {
    api_version = 1,
    id = "BRM_025",
    name = "Volcanic Drake",
    text = "Costs (1) less for each minion that died this turn.",
    set = "BRM",
    type = "minion",
    rarity = "common",
    cost = 6,
    attack = 6,
    health = 4,
    tags = { "dragon" },
    auras = {
        {
            active_zones = { "hand" },
            cost = function(ctx, self) return -deaths_this_turn(ctx, self) end,
            targets = function(ctx, self) return { self } end,
        },
    },
}
