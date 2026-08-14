local card = {
    api_version = 1,
    id = "GVG_028",
    name = "Trade Prince Gallywix",
    text = "Whenever your opponent casts a spell, gain a copy of it and give them a Coin.",
    set = "GVG",
    type = "minion",
    class = "rogue",
    rarity = "legendary",
    cost = 6,
    attack = 5,
    health = 8,
    triggers = {
        {
            event = "spell_cast",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:opponent(ctx:controller(self)) and event.player_cast
                    and ctx:entity(event.entity).card_id ~= "GVG_028t"
            end,
            effect = function(ctx, self, event)
                ctx:give_card(ctx:controller(self), ctx:entity(event.entity).card_id)
                ctx:give_card(event.player, "GVG_028t")
            end,
        },
    },
}

card.tokens = {
    {
        id = "GVG_028t",
        name = "Gallywix's Coin",
        text = "Gain 1 Mana Crystal this turn only.\n<i>(Won't trigger Gallywix.)</i>",
        set = "GVG",
        type = "spell",
        cost = 0,
        collectible = false,
        on_play = function(ctx, self)
            ctx:gain_temporary_mana(ctx:controller(self), 1)
        end,
    },
}

return card
