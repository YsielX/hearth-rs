local KEY = "frost_plague_surcharge"

local function pending(ctx, self)
    return ctx:get_player_data(ctx:controller(self), KEY)
end

local function consume(ctx, self, event)
    return event.player == ctx:controller(self) and pending(ctx, self) > 0
end

local function clear(ctx, self)
    local player = ctx:controller(self)
    ctx:set_player_data(player, KEY, 0)
    ctx:disable_player_keyword(player, KEY)
end

return {
    api_version = 1,
    module_type = "keyword",
    id = KEY,
    name = "Frost Plague Surcharge",
    auras = {{
        active_zones = { "hero" },
        cost = pending,
        cost_cap = 10,
        targets = function(ctx, self)
            return ctx:hand(ctx:controller(self))
        end,
    }},
    triggers = {
        { event = "card_played", timing = "after", active_zones = { "hero" }, condition = consume, effect = clear },
        { event = "card_countered", timing = "after", active_zones = { "hero" }, condition = consume, effect = clear },
    },
}
