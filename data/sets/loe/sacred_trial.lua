local card = {
    api_version = 1,
    id = "LOE_027",
    name = "Sacred Trial",
    text = "<b>Secret:</b> After your opponent has at least 3 minions and plays another, destroy it.",
    set = "LOE",
    type = "spell",
    class = "paladin",
    rarity = "common",
    spell_school = "holy",
    cost = 1,
    keywords = { "secret" },
}

card.triggers = {
    {
        event = "card_played",
        timing = "before",
        active_zones = { "secret" },
        condition = function(ctx, self, event)
            return event.player == ctx:opponent(ctx:controller(self))
                and ctx:entity(event.entity).type == "minion"
                and #ctx:board(event.player) >= 3
        end,
        effect = function(ctx, self, event)
            ctx:set_data(self, "trial_minion", event.entity)
        end,
    },
    {
        event = "minion_played",
        timing = "after",
        active_zones = { "secret" },
        condition = function(ctx, self, event)
            return event.entity == ctx:get_data(self, "trial_minion")
        end,
        effect = function(ctx, self, event)
            ctx:set_data(self, "trial_minion", 0)
            ctx:reveal_secret(self)
            ctx:destroy(event.entity)
        end,
    },
}

return card
