return {
    api_version = 1,
    id = "AT_098",
    name = "Sideshow Spelleater",
    text = "<b>Battlecry:</b> Copy your opponent's Hero Power.",
    set = "TGT",
    type = "minion",
    rarity = "epic",
    cost = 6,
    attack = 6,
    health = 5,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        local enemy_power = ctx:player(ctx:opponent(player)).hero_power
        ctx:replace_hero_power(player, ctx:entity(enemy_power).card_id)
    end,
}
