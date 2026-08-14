local function buff_cthun(ctx, player, amount)
    ctx:grant_player_keyword(player, "cthun_buffs")
    ctx:increment_player_data(player, "cthun_attack_buff", amount)
    ctx:increment_player_data(player, "cthun_health_buff", amount)
end

return {
    api_version = 1, id = "OG_284", name = "Twilight Geomancer",
    text = "<b>Taunt</b>\n<b>Battlecry:</b> Give your\nC'Thun +1/+1 and <b>Taunt</b> <i>(wherever it is)</i>.",
    set = "OG", type = "minion", rarity = "common", cost = 2, attack = 1, health = 4,
    keywords = { "taunt", "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        buff_cthun(ctx, player, 1)
        ctx:grant_player_keyword(player, "cthun_taunt")
    end,
}
