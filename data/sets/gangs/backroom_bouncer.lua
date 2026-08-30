return {
    api_version = 1, id = "CFM_658", name = "Backroom Bouncer",
    text = "Whenever a friendly minion dies, gain +1 Attack.",
    set = "GANGS", type = "minion", rarity = "rare", cost = 4, attack = 4, health = 4,
    triggers = {{
        event = "entity_died", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and ctx:entity(event.entity).type == "minion"
        end,
        effect = function(ctx, self) cardlib.effects.buff(ctx, self, 1, 0) end,
    }},
}
