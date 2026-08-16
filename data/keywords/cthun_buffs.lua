local function cthuns(ctx, player)
    local result = {}
    for _, zone in ipairs({ ctx:hand(player), ctx:deck(player), ctx:board(player), ctx:graveyard(player) }) do
        for _, entity in ipairs(zone) do
            if ctx:entity(entity).card_id == "OG_280" then result[#result + 1] = entity end
        end
    end
    return result
end

local function add_stat(ctx, entity, stat, amount)
    if amount == 0 then return end
    cardlib.effects.modify(ctx, entity, {
        stat = stat,
        operation = "add",
        value = amount,
        silenciable = false,
    })
end

local function applied_key(stat)
    return "cthun_" .. stat .. "_applied"
end

local function apply_historical_total(ctx, entity, player)
    if not entity or ctx:entity(entity).card_id ~= "OG_280" then return end
    for _, stat in ipairs({ "attack", "health" }) do
        local total = ctx:get_player_data(player, "cthun_" .. stat .. "_buff") or 0
        local applied = ctx:get_data(entity, applied_key(stat)) or 0
        local missing = total - applied
        if missing > 0 then
            cardlib.effects.modify(ctx, entity, {
                stat = stat,
                operation = "pre_final_add",
                value = missing,
                silenciable = false,
            })
        end
        ctx:set_data(entity, applied_key(stat), total)
    end
end

local function mark_transformed_baseline(ctx, entity)
    if not entity or ctx:entity(entity).card_id ~= "OG_280" then return end
    local player = ctx:controller(entity)
    ctx:set_data(entity, applied_key("attack"), ctx:get_player_data(player, "cthun_attack_buff") or 0)
    ctx:set_data(entity, applied_key("health"), ctx:get_player_data(player, "cthun_health_buff") or 0)
end

return {
    api_version = 1,
    module_type = "keyword",
    id = "cthun_buffs",
    name = "Cthun Buffs",
    triggers = {
        {
            event = "player_script_data_changed",
            timing = "after",
            active_zones = { "hero" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and event.delta ~= 0
                    and (event.key == "cthun_attack_buff" or event.key == "cthun_health_buff")
            end,
            effect = function(ctx, self, event)
                local stat = event.key == "cthun_attack_buff" and "attack" or "health"
                for _, entity in ipairs(cthuns(ctx, event.player)) do
                    add_stat(ctx, entity, stat, event.delta)
                    local key = applied_key(stat)
                    ctx:set_data(entity, key, (ctx:get_data(entity, key) or 0) + event.delta)
                end
            end,
        },
        {
            event = "card_created",
            timing = "after",
            active_zones = { "hero" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and ctx:entity(event.entity).card_id == "OG_280"
            end,
            effect = function(ctx, self, event)
                apply_historical_total(ctx, event.entity, event.player)
            end,
        },
        {
            event = "card_drawn",
            timing = "after",
            active_zones = { "hero" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and ctx:entity(event.entity).card_id == "OG_280"
            end,
            effect = function(ctx, self, event)
                apply_historical_total(ctx, event.entity, event.player)
            end,
        },
        {
            event = "zone_changed",
            timing = "after",
            active_zones = { "hero" },
            condition = function(ctx, self, event)
                return (event.to == "hand" or event.to == "board")
                    and ctx:controller(event.entity) == ctx:controller(self)
                    and ctx:entity(event.entity).card_id == "OG_280"
            end,
            effect = function(ctx, self, event)
                apply_historical_total(ctx, event.entity, ctx:controller(self))
            end,
        },
        {
            event = "minion_summoned",
            timing = "after",
            active_zones = { "hero" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and ctx:entity(event.entity).card_id == "OG_280"
            end,
            effect = function(ctx, self, event)
                apply_historical_total(ctx, event.entity, event.player)
            end,
        },
        {
            event = "transformed",
            timing = "after",
            active_zones = { "hero" },
            condition = function(ctx, self, event)
                return event.to_card == "OG_280"
                    and ctx:controller(event.entity) == ctx:controller(self)
            end,
            effect = function(ctx, self, event)
                mark_transformed_baseline(ctx, event.entity)
            end,
        },
    },
}
