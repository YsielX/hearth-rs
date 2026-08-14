return {
    api_version = 1,
    id = "KAR_205",
    name = "Silverware Golem",
    text = "If you discard this minion, summon it.",
    set = "KARA",
    type = "minion",
    class = "warlock",
    rarity = "rare",
    cost = 3,
    attack = 3,
    health = 4,
    triggers = {{
        event = "card_discarded",
        timing = "after",
        active_zones = { "graveyard" },
        condition = function(ctx, self, event)
            return event.entity == self and event.player == ctx:controller(self)
        end,
        effect = function(ctx, self, event)
            ctx:summon_existing(event.player, self)
        end,
    }},
}
