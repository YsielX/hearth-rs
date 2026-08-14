return {
    api_version = 1, module_type = "keyword", id = "casts_when_drawn", name = "Casts When Drawn",
    triggers = {
        {
            event = "card_drawn", timing = "after", active_zones = { "hand" },
            condition = function(ctx, self, event) return event.entity == self end,
            effect = function(ctx, self, event)
                ctx:cast_existing_spell(self, { skip_if_invalid = true })
                ctx:draw(event.player, 1)
            end,
        },
    },
}
