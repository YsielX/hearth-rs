return {
    api_version = 1,
    id = "AT_109",
    name = "Argent Watchman",
    text = "Can't attack.\n<b>Inspire:</b> Can attack as normal this turn.",
    set = "TGT",
    type = "minion",
    rarity = "rare",
    cost = 2,
    attack = 2,
    health = 4,
    keywords = { "inspire" },
    rules = {
        can_attack = function(ctx, self, current)
            return current and ctx:get_data(self, "can_attack_this_turn") == 1
        end,
    },
    on_inspire = function(ctx, self) ctx:set_data(self, "can_attack_this_turn", 1) end,
    triggers = {
        {
            event = "turn_ended",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self) ctx:set_data(self, "can_attack_this_turn", 0) end,
        },
    },
}
