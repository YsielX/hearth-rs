return {
    api_version = 1, id = "AT_091", name = "Tournament Medic",
    text = "<b>Inspire:</b> Restore #2 Health to your hero.",
    set = "TGT", type = "minion", rarity = "common", cost = 4, attack = 1, health = 8,
    keywords = { "inspire" },
    on_inspire = function(ctx, self)
        cardlib.effects.heal(ctx, ctx:player(ctx:controller(self)).hero, 2)
    end,
}
