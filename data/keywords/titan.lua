return {
    api_version = 1, module_type = "keyword", id = "titan", name = "Titan",
    rules = {
        can_attack = function(ctx, self, current)
            return current
                and ctx:get_data(self, "titan_abilities_used") == 7
                and ctx:get_data(self, "titan_uses_this_turn") == 0
        end,
    },
    required_card_actions = { "titan_1", "titan_2", "titan_3" },
    actions = {
        titan_1 = {
            active_zones = { "board" }, cost = 0,
            condition = function(ctx, self)
                return not ctx:entity(self).frozen
                    and ctx:get_data(self, "titan_uses_this_turn") == 0
                    and (ctx:get_data(self, "titan_abilities_used") & 1) == 0
            end,
            effect = function(ctx, self)
                ctx:set_data(self, "titan_abilities_used", ctx:get_data(self, "titan_abilities_used") | 1)
                ctx:set_data(self, "titan_uses_this_turn", 1)
            end,
        },
        titan_2 = {
            active_zones = { "board" }, cost = 0,
            condition = function(ctx, self)
                return not ctx:entity(self).frozen
                    and ctx:get_data(self, "titan_uses_this_turn") == 0
                    and (ctx:get_data(self, "titan_abilities_used") & 2) == 0
            end,
            effect = function(ctx, self)
                ctx:set_data(self, "titan_abilities_used", ctx:get_data(self, "titan_abilities_used") | 2)
                ctx:set_data(self, "titan_uses_this_turn", 1)
            end,
        },
        titan_3 = {
            active_zones = { "board" }, cost = 0,
            condition = function(ctx, self)
                return not ctx:entity(self).frozen
                    and ctx:get_data(self, "titan_uses_this_turn") == 0
                    and (ctx:get_data(self, "titan_abilities_used") & 4) == 0
            end,
            effect = function(ctx, self)
                ctx:set_data(self, "titan_abilities_used", ctx:get_data(self, "titan_abilities_used") | 4)
                ctx:set_data(self, "titan_uses_this_turn", 1)
            end,
        },
    },
    triggers = {
        {
            event = "turn_started", timing = "after", active_zones = { "board" },
            condition = function(ctx, self, event) return event.player == ctx:controller(self) end,
            effect = function(ctx, self, event) ctx:set_data(self, "titan_uses_this_turn", 0) end,
        },
    },
}
