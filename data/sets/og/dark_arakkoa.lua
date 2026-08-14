local function buff_cthun(ctx, player, amount)
    ctx:grant_player_keyword(player, "cthun_buffs")
    ctx:increment_player_data(player, "cthun_attack_buff", amount)
    ctx:increment_player_data(player, "cthun_health_buff", amount)
end

return {
    api_version = 1,
    id = "OG_293",
    name = "Dark Arakkoa",
    text = "[x]<b>Taunt</b>\n<b>Battlecry:</b> Give your C'Thun\n+4/+4 <i>(wherever it is).</i>",
    set = "OG",
    type = "minion",
    class = "druid",
    rarity = "common",
    cost = 6,
    attack = 5,
    health = 7,
    keywords = { "taunt", "battlecry" },
    on_battlecry = function(ctx, self)
        buff_cthun(ctx, ctx:controller(self), 4)
    end,
}
