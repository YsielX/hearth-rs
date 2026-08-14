local card = {
    api_version = 1, id = "OG_318", name = "Hogger, Doom of Elwynn",
    text = "Whenever this minion takes damage, summon a 2/2 Gnoll with <b>Taunt</b>.", set = "OG",
    type = "minion", rarity = "legendary", cost = 7, attack = 6, health = 6,
    triggers = {{
        event = "damaged", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.target == self and event.amount > 0 end,
        effect = function(ctx, self) ctx:summon(ctx:controller(self), "OG_318t") end,
    }},
}
card.tokens = {{ id = "OG_318t", name = "Gnoll", text = "<b>Taunt</b>", set = "OG",
    type = "minion", cost = 2, attack = 2, health = 2, keywords = { "taunt" } }}
return card
