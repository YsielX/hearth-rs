return {
    api_version = 1,
    id = "CFM_095",
    name = "Weasel Tunneler",
    text = "<b>Deathrattle:</b> Shuffle this minion into your opponent's deck.",
    set = "GANGS",
    type = "minion",
    rarity = "epic",
    cost = 1,
    attack = 1,
    health = 1,
    tags = { "beast" },
    keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        ctx:shuffle_entity_into_deck(ctx:opponent(ctx:controller(self)), self)
    end,
}
