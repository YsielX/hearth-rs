local function buff_cthun(ctx, player, amount)
    ctx:grant_player_keyword(player, "cthun_buffs")
    ctx:increment_player_data(player, "cthun_attack_buff", amount)
    ctx:increment_player_data(player, "cthun_health_buff", amount)
end

return {
    api_version = 1, id = "OG_302", name = "Usher of Souls",
    text = "Whenever a minion dies, give your C'Thun +1/+1\n<i>(wherever it is).</i>", set = "OG",
    type = "minion", class = "warlock", rarity = "common", cost = 4, attack = 3, health = 5,
    triggers = {{
        event = "entity_died", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return ctx:entity(event.entity).type == "minion" end,
        effect = function(ctx, self) buff_cthun(ctx, ctx:controller(self), 1) end,
    }},
}
