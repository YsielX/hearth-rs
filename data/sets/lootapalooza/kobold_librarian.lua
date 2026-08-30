return {
    api_version = 1,
    id = "LOOT_014", rarity = "common",
    name = "Kobold Librarian",
    text = "<b>Battlecry:</b> Draw a card. Deal 2 damage to your hero.",
    set = "LOOTAPALOOZA",
    type = "minion",
    class = "warlock",
    cost = 1,
    attack = 2,
    health = 1,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        ctx:draw(player, 1)
        cardlib.effects.damage(ctx, ctx:player(player).hero, 2)
    end,
}
