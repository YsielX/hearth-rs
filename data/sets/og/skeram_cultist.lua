local function buff_cthun(ctx, player)
    ctx:grant_player_keyword(player, "cthun_buffs")
    ctx:increment_player_data(player, "cthun_attack_buff", 2)
    ctx:increment_player_data(player, "cthun_health_buff", 2)
end

return {
    api_version = 1, id = "OG_339", name = "Skeram Cultist",
    text = "[x]<b>Battlecry:</b> Give your C'Thun\n+2/+2 <i>(wherever it is)</i>.", set = "OG",
    type = "minion", rarity = "rare", cost = 6, attack = 7, health = 6,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self) buff_cthun(ctx, ctx:controller(self)) end,
}
