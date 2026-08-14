return {
    api_version = 1, module_type = "keyword", id = "echo", name = "Echo",
    hooks = {
        on_play = function(ctx, self)
            ctx:give_card(ctx:controller(self), ctx:entity(self).card_id)
        end,
    },
    rules = {
        minimum_cost = function(ctx, self, current)
            if ctx:get_data(self, "echo_copy") == 1 then
                return math.max(current, 1)
            end
            return current
        end,
    },
    triggers = {
        {
            event = "card_created", timing = "after", active_zones = { "hand" },
            condition = function(ctx, self, event)
                return event.entity == self
                    and event.source ~= nil
                    and ctx:entity(event.source).card_id == ctx:entity(self).card_id
            end,
            effect = function(ctx, self, event)
                ctx:set_data(self, "echo_copy", 1)
                ctx:modify(self, {
                    stat = "cost", operation = "set",
                    value = ctx:entity(event.source).cost,
                })
                ctx:grant_keyword(self, "temporary")
            end,
        },
    },
}
