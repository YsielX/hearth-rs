local ACTIVE = "next_murloc_health_cost"
local PENDING = "next_murloc_health_cost_pending"

local function is_murloc(ctx, entity)
    local snapshot = ctx:entity(entity)
    if snapshot.type ~= "minion" then return false end
    for _, tag in ipairs(ctx:card_definition(snapshot.card_id).tags or {}) do
        if tag == "murloc" or tag == "all" then return true end
    end
    return false
end

local function consume(ctx, self, event)
    local player = ctx:controller(self)
    return event.player == player
        and ctx:get_player_data(player, ACTIVE) > 0
        and is_murloc(ctx, event.entity)
end

local function consume_one(ctx, self)
    local player = ctx:controller(self)
    local remaining = math.max(0, ctx:get_player_data(player, ACTIVE) - 1)
    ctx:set_player_data(player, ACTIVE, remaining)
    if remaining == 0 and ctx:get_player_data(player, PENDING) == 0 then
        ctx:disable_player_keyword(player, "next_murloc_costs_health")
    end
end

return {
    api_version = 1, module_type = "keyword", id = "next_murloc_costs_health",
    name = "Next Murloc Costs Health",
    auras = {{
        active_zones = { "hero" }, keywords = { "costs_health_instead_of_mana" },
        targets = function(ctx, self)
            local player = ctx:controller(self)
            local result = {}
            if ctx:get_player_data(player, ACTIVE) == 0 then return result end
            for _, entity in ipairs(ctx:hand(player)) do
                if is_murloc(ctx, entity) then result[#result + 1] = entity end
            end
            return result
        end,
    }},
    triggers = {
        { event = "card_played", timing = "after", active_zones = { "hero" }, condition = consume, effect = consume_one },
        { event = "card_countered", timing = "after", active_zones = { "hero" }, condition = consume, effect = consume_one },
        {
            event = "turn_ended", timing = "after", active_zones = { "hero" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and (ctx:get_player_data(event.player, ACTIVE) > 0
                        or ctx:get_player_data(event.player, PENDING) > 0)
            end,
            effect = function(ctx, self, event)
                ctx:set_player_data(event.player, ACTIVE, 0)
                ctx:set_player_data(event.player, PENDING, 0)
                ctx:disable_player_keyword(event.player, "next_murloc_costs_health")
            end,
        },
    },
}
