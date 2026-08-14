return {
    api_version = 1, module_type = "keyword", id = "summoned_when_drawn", name = "Summoned When Drawn",
    triggers = {
        {
            event = "card_drawn", timing = "after", active_zones = { "hand" },
            condition = function(ctx, self, event)
                return event.entity == self and ctx:entity(self).type == "minion"
            end,
            effect = function(ctx, self, event)
                ctx:summon_from_hand(self)
                ctx:draw(event.player, 1)
            end,
        },
    },
}
