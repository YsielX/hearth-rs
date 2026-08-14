return {
    api_version = 1, id = "GVG_113", name = "Foe Reaper 4000",
    text = "Also damages the minions next to whomever it attacks.", set = "GVG", type = "minion",
    rarity = "legendary", cost = 8, attack = 6, health = 9, tags = { "mech" },
    triggers = {{
        event = "attack", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.attacker == self and ctx:entity(event.defender).type == "minion"
        end,
        effect = function(ctx, self, event)
            ctx:damage_all(ctx:adjacent_minions(event.defender), ctx:entity(self).attack)
        end,
    }},
}
