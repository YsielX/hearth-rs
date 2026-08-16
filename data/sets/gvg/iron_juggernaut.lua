local function shuffle_mine(ctx, self)
    ctx:shuffle_card_into_deck(ctx:opponent(ctx:controller(self)), "GVG_056t")
end

local card = {
    api_version = 1,
    id = "GVG_056",
    name = "Iron Juggernaut",
    text = "[x]<b>Battlecry and Deathrattle:</b>\nShuffle a Mine into your\nopponent's deck. When drawn,\nexplode for 10 damage!",
    set = "GVG",
    type = "minion",
    class = "warrior",
    rarity = "legendary",
    cost = 6,
    attack = 6,
    health = 5,
    tags = { "mech" },
    keywords = { "battlecry", "deathrattle" },
}

card.on_battlecry = shuffle_mine
card.on_deathrattle = shuffle_mine

card.tokens = {
    {
        id = "GVG_056t",
        name = "Burrowing Mine",
        text = "<b>Casts When Drawn</b>\nYou take 10 damage.",
        set = "GVG",
        type = "spell",
        class = "warrior",
        cost = 6,
        keywords = { "casts_when_drawn" },
        triggers = {
            {
                event = "damaged",
                timing = "before",
                active_zones = { "graveyard" },
                condition = function(ctx, self, event)
                    return event.source == self
                        and event.target == ctx:player(ctx:controller(self)).hero
                end,
                effect = function(ctx, self, event)
                    -- Burrowing Mine has the official ImmuneToSpellpower mechanic.
                    cardlib.effects.set_event_amount(ctx, event, 10)
                end,
            },
        },
        on_play = function(ctx, self)
            local player = ctx:controller(self)
            cardlib.effects.damage(ctx, ctx:player(player).hero, 10)
        end,
    },
}

return card
