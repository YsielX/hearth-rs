return {
    api_version = 1,
    id = "GVG_011",
    name = "Shrinkmeister",
    text = "<b>Battlecry:</b> Give a minion -3 Attack this turn.",
    set = "GVG",
    type = "minion",
    class = "priest",
    rarity = "common",
    cost = 2,
    attack = 3,
    health = 2,
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx, self)
        return ctx:minions()
    end,
    on_battlecry = function(ctx, self, target)
        if target ~= nil then cardlib.effects.buff_until_end_of_turn(ctx, target, -3, 0) end
    end,
}
