return {
    api_version = 1, id = "OG_300", name = "The Boogeymonster",
    text = "Whenever this attacks and kills a minion, gain +2/+2.", set = "OG", type = "minion",
    rarity = "legendary", cost = 8, attack = 6, health = 7,
    triggers = {{
        event = "attack", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            if event.attacker ~= self then return false end
            local defender = ctx:entity(event.defender)
            return defender.type == "minion" and defender.zone == "graveyard"
        end,
        effect = function(ctx, self) cardlib.effects.buff(ctx, self, 2, 2) end,
    }},
}
