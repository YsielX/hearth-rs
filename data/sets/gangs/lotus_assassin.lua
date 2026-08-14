return {
    api_version = 1, id = "CFM_634", name = "Lotus Assassin",
    text = "<b>Stealth</b>. Whenever this attacks and kills a minion, gain <b>Stealth</b>.",
    set = "GANGS", type = "minion", class = "rogue", rarity = "epic",
    cost = 5, attack = 5, health = 5, keywords = { "stealth" },
    triggers = {{
        event = "attack", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            local defender = ctx:entity(event.defender)
            return event.attacker == self and defender.type == "minion" and defender.zone == "graveyard"
        end,
        effect = function(ctx, self) ctx:grant_keyword(self, "stealth") end,
    }},
}
