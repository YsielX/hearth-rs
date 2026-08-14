local KEY = "next_hero_power_discount"

return {
    api_version = 1, id = "AT_115", name = "Fencing Coach",
    text = "<b>Battlecry:</b> The next time you use your Hero Power, it costs (2) less.",
    set = "TGT", type = "minion", rarity = "rare", cost = 3, attack = 2, health = 2,
    keywords = { "battlecry" }, on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        ctx:set_player_data(player, KEY, ctx:get_player_data(player, KEY) + 2)
        ctx:grant_player_keyword(player, KEY)
    end,
}
