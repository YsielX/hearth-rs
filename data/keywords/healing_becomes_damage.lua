return {
    api_version = 1,
    module_type = "keyword",
    id = "healing_becomes_damage",
    name = "Healing Becomes Damage",
    rules = {
        healing_becomes_damage = function(ctx, self, current) return true end,
    },
    triggers = {{
        event = "turn_ended",
        timing = "after",
        active_zones = { "hero" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self)
        end,
        effect = function(ctx, self)
            ctx:disable_player_keyword(ctx:controller(self), "healing_becomes_damage")
        end,
    }},
}
