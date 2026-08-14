local function buff_cthun(ctx, player)
    ctx:grant_player_keyword(player, "cthun_buffs")
    ctx:increment_player_data(player, "cthun_attack_buff", 1)
    ctx:increment_player_data(player, "cthun_health_buff", 1)
end

return {
    api_version = 1, id = "OG_321", name = "Crazed Worshipper",
    text = "[x]<b>Taunt</b>\nWhenever this minion takes\ndamage, give your C'Thun\n+1/+1 <i>(wherever it is).</i>",
    set = "OG", type = "minion", rarity = "epic", cost = 4, attack = 3, health = 6,
    keywords = { "taunt" }, triggers = {{
        event = "damaged", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.target == self and event.amount > 0 end,
        effect = function(ctx, self) buff_cthun(ctx, ctx:controller(self)) end,
    }},
}
