return {
    api_version = 1,
    id = "CFM_060",
    name = "Red Mana Wyrm",
    text = "Whenever you cast a spell, gain +2 Attack.",
    set = "GANGS",
    type = "minion",
    rarity = "common",
    cost = 5,
    attack = 2,
    health = 6,
    triggers = {{
        event = "spell_cast", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.player_cast
        end,
        effect = function(ctx, self) cardlib.effects.buff(ctx, self, 2, 0) end,
    }},
}
