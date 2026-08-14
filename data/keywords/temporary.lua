return {
    api_version = 1, module_type = "keyword", id = "temporary", name = "Temporary",
    triggers = {
        {
            event = "turn_ended", timing = "after", active_zones = { "hand" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self, event)
                -- Temporary cards disappear; this is not a discard and must not
                -- activate discard triggers.
                ctx:move(self, "removed")
            end,
        },
    },
}
