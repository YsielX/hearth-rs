return {
    api_version = 1, id = "UNG_803", name = "Emerald Reaver",
    text = "<b>Battlecry:</b> Deal 1 damage to each hero.",
    set = "UNGORO", type = "minion", rarity = "common", cost = 1, attack = 2, health = 1,
    tags = { "beast" }, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        cardlib.effects.damage_all(ctx, { ctx:player(0).hero, ctx:player(1).hero }, 1)
    end,
}
