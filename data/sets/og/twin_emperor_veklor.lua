local function cthun_attack(ctx, player)
    local value = 6 + (ctx:get_player_data(player, "cthun_attack_buff") or 0)
    for _, zone in ipairs({ ctx:hand(player), ctx:deck(player), ctx:board(player), ctx:graveyard(player) }) do
        for _, entity in ipairs(zone) do
            if ctx:entity(entity).card_id == "OG_280" then value = math.max(value, ctx:entity(entity).attack) end
        end
    end
    return value
end

local card = {
    api_version = 1,
    id = "OG_131",
    name = "Twin Emperor Vek'lor",
    text = "[x]<b><b>Taunt</b>\nBattlecry:</b> If your C'Thun has\nat least 10 Attack, summon\nanother Emperor.",
    set = "OG",
    type = "minion",
    rarity = "legendary",
    cost = 7,
    attack = 6,
    health = 7,
    keywords = { "taunt", "battlecry" },
    tokens = {{
        api_version = 1,
        id = "OG_319",
        name = "Twin Emperor Vek'nilash",
        text = "<b>Taunt</b>",
        set = "OG",
        type = "minion",
        cost = 7,
        attack = 6,
        health = 7,
        keywords = { "taunt" },
    }},
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    if cthun_attack(ctx, player) >= 10 then
        ctx:summon(player, "OG_319")
    end
end

return card
