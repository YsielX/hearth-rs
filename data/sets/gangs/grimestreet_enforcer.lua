return {
    api_version = 1, id = "CFM_639", name = "Grimestreet Enforcer",
    text = "At the end of your turn, give all minions in your hand +1/+1.",
    set = "GANGS", type = "minion", class = "paladin", rarity = "rare",
    cost = 4, attack = 4, health = 4,
    triggers = {{
        event = "turn_ended", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.player == ctx:controller(self) end,
        effect = function(ctx, self)
            for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
                if ctx:entity(entity).type == "minion" then cardlib.effects.buff(ctx, entity, 1, 1) end
            end
        end,
    }},
}
