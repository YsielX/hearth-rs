local function minions_died_this_turn(ctx, self)
    local player = ctx:controller(self)
    return #ctx:minions_died_this_turn(player)
        + #ctx:minions_died_this_turn(ctx:opponent(player))
end

return {
    api_version = 1,
    id = "BRM_001",
    name = "Solemn Vigil",
    text = "Draw 2 cards. Costs (1) less for each minion that died this turn.",
    set = "BRM",
    type = "spell",
    class = "paladin",
    rarity = "common",
    cost = 5,
    on_play = function(ctx, self) ctx:draw(ctx:controller(self), 2) end,
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
