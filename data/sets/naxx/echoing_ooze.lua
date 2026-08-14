return {
    api_version = 1,
    id = "FP1_003",
    name = "Echoing Ooze",
    text = "<b>Battlecry:</b> Summon an exact copy of this minion at the end of the turn.",
    set = "NAXX",
    type = "minion",
    rarity = "epic",
    cost = 2,
    attack = 1,
    health = 2,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        ctx:set_data(self, "echo_turn", ctx:turn())
    end,
    triggers = {
        {
            event = "turn_ended",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and ctx:get_data(self, "echo_turn") == event.turn
            end,
            effect = function(ctx, self)
                ctx:summon_copy(ctx:controller(self), self)
            end,
        },
    },
}
