return {
    api_version = 1, id = "ICC_031", name = "Night Howler",
    text = "Whenever this minion takes\ndamage, gain +2 Attack.",
    set = "ICECROWN", type = "minion", rarity = "common", cost = 4, attack = 3, health = 4,
    triggers = {{
        event = "damaged", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.target == self and event.amount > 0 and ctx:entity(self).health > 0
        end,
        effect = function(ctx, self) ctx:buff(self, 2, 0) end,
    }},
}
