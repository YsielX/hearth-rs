return {
    api_version = 1,
    id = "LOE_111",
    name = "Excavated Evil",
    text = "Deal $3 damage to all minions.\nShuffle this card into your opponent's deck.",
    set = "LOE",
    type = "spell",
    class = "priest",
    rarity = "rare",
    spell_school = "shadow",
    cost = 5,
    on_play = function(ctx, self)
        cardlib.effects.damage_all(ctx, ctx:minions(), 3)
        ctx:shuffle_entity_into_deck(ctx:opponent(ctx:controller(self)), self)
    end,
}
