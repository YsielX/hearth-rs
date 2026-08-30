return {
    api_version = 1,
    id = "TTN_083",
    name = "Son of Hodir",
    text = "<b>Battlecry:</b> Shuffle four\n8/8 Giants into your deck that are summoned when drawn.",
    set = "TITANS",
    type = "minion",
    rarity = "epic",
    class = "neutral",
    cost = 8,
    attack = 8,
    health = 8,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        for _ = 1, 4 do cardlib.effects.shuffle_card_into_deck(ctx, player, "TTN_083t") end
    end,
    tokens = {
        {
            id = "TTN_083t",
            name = "Frost Tyrant",
            text = "<b>Summoned When Drawn</b>",
            set = "TITANS",
            type = "minion",
            class = "neutral",
            cost = 8,
            attack = 8,
            health = 8,
            keywords = { "summoned_when_drawn" },
        },
    },
}
