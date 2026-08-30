local KEY = "mograine_end_turn_damage"

return {
    api_version = 1,
    module_type = "keyword",
    id = "mograine",
    name = "Mograine's Curse",
    triggers = {{
        event = "turn_ended",
        timing = "after",
        active_zones = { "hero" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self)
                and ctx:get_player_data(event.player, KEY) > 0
        end,
        effect = function(ctx, self)
            local player = ctx:controller(self)
            local enemy = ctx:opponent(player)
            cardlib.effects.damage(
                ctx,
                ctx:player(enemy).hero,
                ctx:get_player_data(player, KEY)
            )
        end,
    }},
}
