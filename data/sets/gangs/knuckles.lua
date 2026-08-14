return {
    api_version = 1,
    id = "CFM_333",
    name = "Knuckles",
    text = "After this attacks a\nminion, it also hits the enemy hero.",
    set = "GANGS",
    type = "minion",
    class = "hunter",
    rarity = "legendary",
    cost = 5,
    attack = 3,
    health = 7,
    tags = { "beast" },
    triggers = {{
        event = "attack", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.attacker == self and ctx:entity(event.defender).type == "minion"
        end,
        effect = function(ctx, self)
            local enemy = ctx:opponent(ctx:controller(self))
            ctx:damage(ctx:player(enemy).hero, ctx:entity(self).attack)
        end,
    }},
}
