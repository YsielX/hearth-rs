local KEY = "next_secret_cost_one_this_turn"

local function is_secret(ctx, entity)
    for _, keyword in ipairs(ctx:card_definition(ctx:entity(entity).card_id).keywords or {}) do
        if keyword == "secret" then return true end
    end
    return false
end

local function consume(ctx, self, event)
    local player = ctx:controller(self)
    return event.player == player
        and ctx:get_player_data(player, KEY) == 1
        and is_secret(ctx, event.entity)
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
    name = "Next Secret Cost One This Turn",
    auras = {{
        active_zones = { "hero" },
        cost_set = 1,
        targets = function(ctx, self)
            local result = {}
            for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
                if is_secret(ctx, entity) then result[#result + 1] = entity end
            end
            return result
        end,
    }},
    triggers = {
        { event = "card_played", timing = "after", active_zones = { "hero" }, condition = consume, effect = clear },
        { event = "card_countered", timing = "after", active_zones = { "hero" }, condition = consume, effect = clear },
        {
            event = "turn_ended", timing = "after", active_zones = { "hero" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and ctx:get_player_data(ctx:controller(self), KEY) == 1
            end,
            effect = clear,
        },
    },
}
