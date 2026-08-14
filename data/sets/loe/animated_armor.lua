return {
    api_version = 1, id = "LOE_119", name = "Animated Armor",
    text = "Your hero can only take 1 damage at a time.",
    set = "LOE", type = "minion", class = "mage", rarity = "rare",
    cost = 4, attack = 4, health = 4,
    triggers = {{
        event = "damaged", timing = "before", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.target == ctx:player(ctx:controller(self)).hero and event.amount > 1
        end,
        effect = function(ctx, self, event) ctx:set_event_amount(event, 1) end,
    }},
}
