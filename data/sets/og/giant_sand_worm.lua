local card = {
    api_version = 1, id = "OG_308", name = "Giant Sand Worm",
    text = "Whenever this attacks and kills a minion, it may attack again.",
    set = "OG", type = "minion", class = "hunter", rarity = "epic",
    cost = 8, attack = 8, health = 8, tags = { "beast" },
    rules = {
        max_attacks = function(ctx, self, current)
            return current + ctx:get_data(self, "extra_attacks_after_kills")
        end,
    },
}
card.triggers = {
    {
        event = "attack", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            if event.attacker ~= self then return false end
            local defender = ctx:entity(event.defender)
            return defender.type == "minion" and defender.zone == "graveyard"
        end,
        effect = function(ctx, self)
            ctx:set_data(self, "extra_attacks_after_kills",
                ctx:get_data(self, "extra_attacks_after_kills") + 1)
        end,
    },
    {
        event = "turn_ended", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self)
        end,
        effect = function(ctx, self)
            ctx:set_data(self, "extra_attacks_after_kills", 0)
        end,
    },
}
return card
