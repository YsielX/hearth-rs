local function weapons(ctx)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        if ctx:card_definition(card_id).type == "weapon" then result[#result + 1] = card_id end
    end
    return result
end

local card = {
    api_version = 1, id = "OG_220", name = "Malkorok",
    text = "<b>Battlecry:</b> Equip a random weapon.", set = "OG", type = "minion",
    class = "warrior", rarity = "legendary", cost = 7, attack = 6, health = 5,
    keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    local pool = weapons(ctx)
    if #pool > 0 then ctx:random_value(pool, "equip_random_weapon") end
end
function card.equip_random_weapon(ctx, self, card_id)
    ctx:equip_weapon(ctx:controller(self), card_id)
end
return card
