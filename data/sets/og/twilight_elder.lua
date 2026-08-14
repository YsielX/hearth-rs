local function buff_cthun(ctx, player)
    ctx:grant_player_keyword(player, "cthun_buffs")
    ctx:increment_player_data(player, "cthun_attack_buff", 1)
    ctx:increment_player_data(player, "cthun_health_buff", 1)
end

return {
    api_version = 1, id = "OG_286", name = "Twilight Elder",
    text = "At the end of your turn, give your C'Thun +1/+1 <i>(wherever it is).</i>", set = "OG",
    type = "minion", rarity = "common", cost = 3, attack = 3, health = 4,
    triggers = {{
        event = "turn_ended", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.player == ctx:controller(self) end,
        effect = function(ctx, self) buff_cthun(ctx, ctx:controller(self)) end,
    }},
}
