local card = {
    api_version = 1, id = "AT_017", name = "Twilight Guardian",
    text = "<b>Battlecry:</b> If you're holding a Dragon, gain +1 Attack and <b>Taunt</b>.",
    set = "TGT", type = "minion", rarity = "epic", cost = 4, attack = 2, health = 6,
    tags = { "dragon" }, keywords = { "battlecry" },
}

local function is_dragon(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags) do
        if tag == "dragon" or tag == "all" then return true end
    end
    return false
end

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    for _, entity in ipairs(ctx:hand(player)) do
        if is_dragon(ctx, entity) then
            cardlib.effects.buff(ctx, self, 1, 0)
            cardlib.effects.grant_keyword(ctx, self, "taunt")
            return
        end
    end
end

return card
