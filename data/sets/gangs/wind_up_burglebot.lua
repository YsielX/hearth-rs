return {
    api_version = 1,
    id = "CFM_025",
    name = "Wind-up Burglebot",
    text = "Whenever this attacks a minion and survives, draw a card.",
    set = "GANGS",
    type = "minion",
    rarity = "epic",
    cost = 6,
    attack = 5,
    health = 5,
    tags = { "mech" },
    triggers = {{
        event = "attack", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.attacker == self
                and ctx:entity(event.defender).type == "minion"
                and ctx:entity(self).health > 0
        end,
        effect = function(ctx, self) ctx:draw(ctx:controller(self), 1) end,
    }},
}
