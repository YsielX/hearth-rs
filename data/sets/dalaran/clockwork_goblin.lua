local card = {
    api_version = 1,
    id = "DAL_060", rarity = "rare",
    name = "Clockwork Goblin",
    text = "[x]<b>Battlecry:</b> Shuffle a Bomb\ninto your opponent's deck.\nWhen drawn, it explodes\nfor 5 damage.",
    set = "DALARAN",
    type = "minion",
    class = "warrior",
    cost = 3,
    attack = 3,
    health = 3,
    tags = { "mech" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    cardlib.effects.shuffle_card_into_deck(ctx, ctx:opponent(ctx:controller(self)), "BOT_511t")
end

card.tokens = {
    {
        id = "BOT_511t",
        name = "Bomb",
        text = "<b>Casts When Drawn</b>\nYou take 5 damage.",
        set = "BOOMSDAY",
        type = "spell",
        class = "neutral",
        collectible = false,
        cost = 5,
        keywords = { "casts_when_drawn" },
        on_play = function(ctx, self)
            local player = ctx:controller(self)
            cardlib.effects.damage(ctx, ctx:player(player).hero, 5)
        end,
    },
}

return card
