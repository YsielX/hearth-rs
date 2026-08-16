return {
    api_version = 1,
    id = "AT_111",
    name = "Refreshment Vendor",
    text = "<b>Battlecry:</b> Restore #4 Health to each hero.",
    set = "TGT",
    type = "minion",
    rarity = "common",
    cost = 4,
    attack = 3,
    health = 5,
    keywords = { "battlecry" },
    on_battlecry = function(ctx)
        cardlib.effects.heal_all(ctx, { ctx:player(0).hero, ctx:player(1).hero }, 4)
    end,
}
