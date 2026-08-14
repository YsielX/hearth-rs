local KEY = "hero_power_next_turn_surcharge"
local COUNT_KEY = KEY .. ":count"
local EXPIRES_KEY = KEY .. ":expires"

return {
    api_version = 1,
    module_type = "keyword",
    id = KEY,
    name = "Hero Power Next Turn Surcharge",
    auras = {{
        active_zones = { "hero" },
        cost = function(ctx, self)
            return 5 * ctx:get_player_data(ctx:controller(self), COUNT_KEY)
        end,
        targets = function(ctx, self)
            return { ctx:player(ctx:controller(self)).hero_power }
        end,
    }},
    triggers = {{
        event = "turn_ended", timing = "after", active_zones = { "hero" },
        condition = function(ctx, self, event)
            local player = ctx:controller(self)
            return event.player == player
                and ctx:get_player_data(player, COUNT_KEY) > 0
                and event.turn >= ctx:get_player_data(player, EXPIRES_KEY)
        end,
        effect = function(ctx, self, event)
            local player = ctx:controller(self)
            ctx:set_player_data(player, COUNT_KEY, 0)
            ctx:set_player_data(player, EXPIRES_KEY, 0)
            ctx:disable_player_keyword(player, KEY)
        end,
    }},
}
