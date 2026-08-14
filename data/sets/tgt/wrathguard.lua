return {
    api_version = 1, id = "AT_026", name = "Wrathguard",
    text = "Whenever this minion takes damage, also deal that amount to your hero.", set = "TGT",
    type = "minion", class = "warlock", rarity = "common", cost = 2, attack = 4, health = 3,
    tags = { "demon" }, triggers = {{
        event = "damaged", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.target == self and event.amount > 0 end,
        effect = function(ctx, self, event)
            ctx:damage(ctx:player(ctx:controller(self)).hero, event.amount)
        end,
    }},
}
