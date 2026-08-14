local function minions_died_this_turn(ctx, self)
    local player = ctx:controller(self)
    return #ctx:minions_died_this_turn(player)
        + #ctx:minions_died_this_turn(ctx:opponent(player))
end

return {
    api_version = 1,
    id = "BRM_009",
    name = "Volcanic Lumberer",
    text = "<b>Taunt</b>\nCosts (1) less for each minion that died this turn.",
    set = "BRM",
    type = "minion",
    class = "druid",
    rarity = "rare",
    cost = 9,
    attack = 7,
    health = 8,
    keywords = { "taunt" },
    auras = {
        {
            active_zones = { "hand", "deck" },
            cost = function(ctx, self)
                return -minions_died_this_turn(ctx, self)
            end,
            targets = function(ctx, self) return { self } end,
        },
    },
}
