return {
    api_version = 1, module_type = "keyword", id = "shatter", name = "Shatter",
    required_card_hooks = { "on_shatter" },
    triggers = {
        {
            event = "card_drawn", timing = "after", active_zones = { "hand" },
            condition = function(ctx, self, event) return event.entity == self end,
            effect = function(ctx, self, event) ctx:continue_with("on_shatter") end,
        },
        {
            event = "card_created", timing = "after", active_zones = { "hand" },
            condition = function(ctx, self, event)
                if event.entity ~= self then return false end
                local source = ctx:card_definition(ctx:entity(event.source).card_id)
                for _, tag in ipairs(source.tags) do
                    if tag == "shatter_fragment" then return false end
                end
                return true
            end,
            effect = function(ctx, self, event) ctx:continue_with("on_shatter") end,
        },
    },
}
