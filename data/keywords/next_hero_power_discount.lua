local KEY = "next_hero_power_discount"

return {
    api_version = 1, module_type = "keyword", id = KEY, name = "Next Hero Power Discount",
    auras = {{
        active_zones = { "hero" },
        cost = function(ctx, self) return -ctx:get_player_data(ctx:controller(self), KEY) end,
        targets = function(ctx, self) return { ctx:player(ctx:controller(self)).hero_power } end,
    }},
    triggers = {{
        event = "hero_power_used", timing = "after", active_zones = { "hero" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and ctx:get_player_data(event.player, KEY) > 0
        end,
        effect = function(ctx, self, event)
            ctx:set_player_data(event.player, KEY, 0)
            ctx:disable_player_keyword(event.player, KEY)
        end,
    }},
}
