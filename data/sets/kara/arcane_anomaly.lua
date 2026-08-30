return {
    api_version = 1,
    id = "KAR_036",
    name = "Arcane Anomaly",
    text = "After you cast a spell, give this minion +1 Health.",
    set = "KARA",
    type = "minion",
    rarity = "common",
    cost = 1,
    attack = 2,
    health = 1,
    tags = { "elemental" },
    triggers = {{
        event = "spell_cast",
        timing = "after",
        active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.player_cast
        end,
        effect = function(ctx, self) cardlib.effects.buff(ctx, self, 0, 1) end,
    }},
}
