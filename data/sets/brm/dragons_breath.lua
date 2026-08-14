local function minions_died_this_turn(ctx, self)
    local player = ctx:controller(self)
    return #ctx:minions_died_this_turn(player)
        + #ctx:minions_died_this_turn(ctx:opponent(player))
end

return {
    api_version = 1,
    id = "BRM_003",
    name = "Dragon's Breath",
    text = "Deal $4 damage. Costs (1) less for each minion that died this turn.",
    set = "BRM",
    type = "spell",
    class = "mage",
    rarity = "common",
    spell_school = "fire",
    cost = 5,
    target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
    on_play = function(ctx, self, target) ctx:damage(target, 4) end,
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
