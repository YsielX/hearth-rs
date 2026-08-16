local function damage_heroes(ctx, self)
    cardlib.effects.damage_all(ctx, { ctx:player(0).hero, ctx:player(1).hero }, 4)
end

return {
    api_version = 1,
    id = "AT_012",
    name = "Spawn of Shadows",
    text = "<b>Battlecry and Inspire:</b>\nDeal 4 damage to\neach hero.",
    set = "TGT",
    type = "minion",
    class = "priest",
    rarity = "rare",
    cost = 5,
    attack = 5,
    health = 5,
    tags = { "undead" },
    keywords = { "battlecry", "inspire" },
    on_battlecry = damage_heroes,
    on_inspire = damage_heroes,
}
