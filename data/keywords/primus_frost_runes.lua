local KEY = "primus_frost_runes"

local function pending(ctx, self)
    return ctx:get_player_data(ctx:controller(self), KEY)
end

local function is_spell(ctx, entity)
    return ctx:entity(entity).type == "spell"
end

local function consume(ctx, self, event)
    return event.player == ctx:controller(self)
        and pending(ctx, self) > 0
        and is_spell(ctx, event.entity)
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
    name = "Primus Frost Runes",
    rules = {
        base_spell_damage = function(ctx, self, current)
            return current + 3 * pending(ctx, self)
        end,
    },
    auras = {{
        active_zones = { "hero" },
        cost = function(ctx, self) return -3 * pending(ctx, self) end,
        targets = function(ctx, self)
            local result = {}
            for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
                if is_spell(ctx, entity) then result[#result + 1] = entity end
            end
            return result
        end,
    }},
    triggers = {
        {
            event = "spell_cast", timing = "after", active_zones = { "hero" },
            condition = function(ctx, self, event)
                return event.player_cast and consume(ctx, self, event)
            end,
            effect = clear,
        },
        { event = "card_countered", timing = "after", active_zones = { "hero" }, condition = consume, effect = clear },
    },
}
