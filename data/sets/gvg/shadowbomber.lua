return {
    api_version = 1,
    id = "GVG_009",
    name = "Shadowbomber",
    text = "<b>Battlecry:</b> Deal 3 damage to each hero.",
    set = "GVG",
    type = "minion",
    class = "priest",
    rarity = "epic",
    cost = 1,
    attack = 3,
    health = 1,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        ctx:damage_all({
            ctx:player(player).hero,
            ctx:player(ctx:opponent(player)).hero,
        }, 3)
    end,
}
