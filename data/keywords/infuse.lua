return {
    api_version = 1, module_type = "keyword", id = "infuse", name = "Infuse",
    requires_param = true,
    required_card_hooks = { "on_infuse" },
    triggers = {
        {
            event = "entity_died", timing = "after", active_zones = { "hand" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self, event)
                local progress = ctx:get_data(self, "infuse_progress") + 1
                local required = ctx:keyword_param(self, "infuse")
                ctx:set_data(self, "infuse_progress", progress)
                if progress >= required then
                    ctx:disable_keyword(self, "infuse")
                    ctx:continue_with("on_infuse")
                end
            end,
        },
    },
}
