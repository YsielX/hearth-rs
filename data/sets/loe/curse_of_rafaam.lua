local card = {
    api_version = 1,
    id = "LOE_007",
    name = "Curse of Rafaam",
    text = "Give your opponent a 'Cursed!' card.\nWhile they hold it, they take 2 damage on their turn.",
    set = "LOE",
    type = "spell",
    class = "warlock",
    rarity = "common",
    cost = 2,
    spell_school = "shadow",
}

function card.on_play(ctx, self)
    ctx:give_card(ctx:opponent(ctx:controller(self)), "LOE_007t")
end

card.tokens = {
    {
        id = "LOE_007t",
        name = "Cursed!",
        text = "While this is in your hand, take 2 damage at the start of your turn.",
        set = "LOE",
        type = "spell",
        class = "warlock",
        cost = 2,
        spell_school = "shadow",
        triggers = {
            {
                event = "turn_started",
                timing = "after",
                active_zones = { "hand" },
                condition = function(ctx, self, event)
                    return event.player == ctx:controller(self)
                end,
                effect = function(ctx, self)
                    local player = ctx:controller(self)
                    ctx:damage(ctx:player(player).hero, 2)
                end,
            },
        },
    },
}

return card
