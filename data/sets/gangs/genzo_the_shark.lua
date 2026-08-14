return {
    api_version = 1, id = "CFM_808", name = "Genzo, the Shark",
    text = "Whenever this attacks, both players draw until they have 3 cards.",
    set = "GANGS", type = "minion", rarity = "legendary", cost = 4,
    attack = 5, health = 4, tags = { "undead" },
    triggers = {{
        event = "attack", timing = "before", active_zones = { "board" },
        condition = function(ctx, self, event) return event.attacker == self end,
        effect = function(ctx, self)
            local player = ctx:controller(self)
            local opponent = ctx:opponent(player)
            ctx:draw(player, math.max(0, 3 - #ctx:hand(player)))
            ctx:draw(opponent, math.max(0, 3 - #ctx:hand(opponent)))
        end,
    }},
}
