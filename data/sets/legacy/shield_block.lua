return {
    api_version = 1,
    id = "EX1_606", rarity = "free",
    name = "Shield Block",
    text = "Gain 5 Armor.\nDraw a card.",
    set = "LEGACY",
    type = "spell",
    class = "warrior",
    cost = 2,
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        ctx:gain_armor(player, 5)
        ctx:draw(player, 1)
    end,
}
