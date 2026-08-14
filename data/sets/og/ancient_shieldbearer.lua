local function cthun_attack(ctx, player)
    local value = 6 + (ctx:get_player_data(player, "cthun_attack_buff") or 0)
    for _, zone in ipairs({ ctx:hand(player), ctx:deck(player), ctx:board(player), ctx:graveyard(player) }) do
        for _, entity in ipairs(zone) do
            if ctx:entity(entity).card_id == "OG_280" then value = math.max(value, ctx:entity(entity).attack) end
        end
    end
    return value
end

return {
    api_version = 1, id = "OG_301", name = "Ancient Shieldbearer",
    text = "<b>Battlecry:</b> If your C'Thun has at least 10 Attack, gain 10 Armor.", set = "OG",
    type = "minion", class = "warrior", rarity = "rare", cost = 6, attack = 6, health = 6,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        if cthun_attack(ctx, player) >= 10 then
            ctx:gain_armor(player, 10)
        end
    end,
}
