return {
    api_version = 1,
    module_type = "hero_power",
    id = "HERO_11bp", rarity = "free",
    name = "Ghoul Charge",
    text = "[x]<b>Hero Power</b>\nSummon a 1/1 Ghoul\nwith <b>Charge</b>. It dies at\nend of turn.",
    set = "BASIC",
    class = "death_knight",
    cost = 2,
    on_play = function(ctx, self)
        ctx:summon(ctx:controller(self), "HERO_11bpt")
    end,
    tokens = {
        {
            id = "HERO_11bpt", rarity = "free", name = "Frail Ghoul",
            text = "[x]<b>Charge</b>\nAt the end of your turn,\nthis minion dies.",
            set = "BASIC", type = "minion", class = "death_knight",
            cost = 1, attack = 1, health = 1, tags = { "undead" },
            keywords = { "charge" },
            triggers = {
                {
                    event = "turn_ended", timing = "after", active_zones = { "board" },
                    condition = function(ctx, self, event)
                        return event.player == ctx:controller(self)
                    end,
                    effect = function(ctx, self) cardlib.effects.destroy(ctx, self) end,
                },
            },
        },
    },
}
