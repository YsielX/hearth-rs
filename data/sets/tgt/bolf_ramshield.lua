return {
    api_version = 1, id = "AT_124", name = "Bolf Ramshield",
    text = "Whenever your hero takes damage, this minion takes it instead.", set = "TGT",
    type = "minion", rarity = "legendary", cost = 6, attack = 3, health = 9,
    triggers = {{
        event = "damaged", timing = "before", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.target == ctx:player(ctx:controller(self)).hero and event.amount > 0
        end,
        effect = function(ctx, self, event) ctx:set_damage_target(event.event_id, self) end,
    }},
}
