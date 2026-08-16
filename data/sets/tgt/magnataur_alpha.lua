return {
    api_version = 1, id = "AT_067", name = "Magnataur Alpha",
    text = "Also damages the minions next to whomever\nhe attacks.", set = "TGT", type = "minion",
    class = "warrior", rarity = "epic", cost = 4, attack = 5, health = 3,
    triggers = {{
        event = "attack", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.attacker == self and ctx:entity(event.defender).type == "minion"
        end,
        effect = function(ctx, self, event)
            cardlib.effects.damage_all(ctx, ctx:adjacent_minions(event.defender), ctx:entity(self).attack)
        end,
    }},
}
