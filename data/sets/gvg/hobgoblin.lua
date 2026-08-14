return {
    api_version = 1, id = "GVG_104", name = "Hobgoblin",
    text = "Whenever you play a 1-Attack minion, give it +2/+2.", set = "GVG",
    type = "minion", rarity = "epic", cost = 3, attack = 2, health = 3,
    triggers = {{
        event = "minion_played", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and ctx:entity(event.entity).attack == 1
        end,
        effect = function(ctx, self, event) ctx:buff(event.entity, 2, 2) end,
    }},
}
