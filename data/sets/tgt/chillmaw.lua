local function holding_dragon(ctx, self)
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
            if tag == "dragon" or tag == "all" then return true end
        end
    end
    return false
end

local card = {
    api_version = 1, id = "AT_123", name = "Chillmaw",
    text = "[x]<b>Taunt</b>\n<b>Deathrattle:</b> If you're holding\na Dragon, deal 3 damage\nto all minions.",
    set = "TGT", type = "minion", rarity = "legendary", cost = 7, attack = 6, health = 6,
    tags = { "dragon", "undead" }, keywords = { "taunt", "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    if holding_dragon(ctx, self) then cardlib.effects.damage_all(ctx, ctx:minions(), 3) end
end

return card
