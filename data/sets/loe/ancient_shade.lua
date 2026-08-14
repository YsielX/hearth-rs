local card = {
    api_version = 1,
    id = "LOE_110",
    name = "Ancient Shade",
    text = "<b>Battlecry:</b> Shuffle an 'Ancient Curse' into your deck that deals 7 damage to you when drawn.",
    set = "LOE",
    type = "minion",
    rarity = "rare",
    cost = 4,
    attack = 7,
    health = 4,
    tags = { "undead" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    ctx:shuffle_card_into_deck(ctx:controller(self), "LOE_110t")
end

card.tokens = {
    {
        id = "LOE_110t",
        name = "Ancient Curse",
        text = "<b>Casts When Drawn</b>\nYou take 7 damage.",
        set = "LOE",
        type = "spell",
        cost = 4,
        spell_school = "shadow",
        keywords = { "casts_when_drawn" },
        on_play = function(ctx, self)
            local player = ctx:controller(self)
            ctx:damage(ctx:player(player).hero, 7)
        end,
    },
}

return card
