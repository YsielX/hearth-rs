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
    api_version = 1,
    id = "OG_188",
    name = "Klaxxi Amber-Weaver",
    text = "[x]<b>Taunt</b>\n<b>Battlecry:</b> If your C'Thun\nhas at least 10 Attack,\ngain +5 Health.",
    set = "OG",
    type = "minion",
    class = "druid",
    rarity = "rare",
    cost = 4,
    attack = 4,
    health = 5,
    keywords = { "taunt", "battlecry" },
    on_battlecry = function(ctx, self)
        if cthun_attack(ctx, ctx:controller(self)) >= 10 then
            ctx:buff(self, 0, 5)
        end
    end,
}
