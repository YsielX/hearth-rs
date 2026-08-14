local ACTIVE_KEY = "dragon_consort_discount"
local PENDING_KEY = "dragon_consort_pending"

local function has_tag(definition, wanted)
    for _, tag in ipairs(definition.tags or {}) do
        if tag == wanted then return true end
    end
    return false
end

local function is_dragon(ctx, entity)
    local snapshot = ctx:entity(entity)
    return snapshot.type == "minion"
        and has_tag(ctx:card_definition(snapshot.card_id), "dragon")
end

local function consume(ctx, self, event)
    local player = ctx:controller(self)
    return event.player == player
        and ctx:get_player_data(player, ACTIVE_KEY) > 0
        and is_dragon(ctx, event.entity)
end

local function consume_discount(ctx, self)
    local player = ctx:controller(self)
    ctx:set_player_data(player, ACTIVE_KEY, 0)
    if ctx:get_player_data(player, PENDING_KEY) == 0 then
        ctx:disable_player_keyword(player, "dragon_consort_discount")
    end
end

return {
    api_version = 1,
    module_type = "keyword",
    id = "dragon_consort_discount",
    name = "Dragon Consort Discount",
    auras = {
        {
            active_zones = { "hero" },
            cost = function(ctx, self)
                return -2 * ctx:get_player_data(ctx:controller(self), ACTIVE_KEY)
            end,
            targets = function(ctx, self)
                local result = {}
                for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
                    if is_dragon(ctx, entity) then result[#result + 1] = entity end
                end
                return result
            end,
        },
    },
    triggers = {
        {
            event = "card_played",
            timing = "after",
            active_zones = { "hero" },
            condition = consume,
            effect = consume_discount,
        },
        {
            event = "card_countered",
            timing = "after",
            active_zones = { "hero" },
            condition = consume,
            effect = consume_discount,
        },
    },
}
