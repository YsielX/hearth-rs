local card = {
    api_version = 1, id = "GVG_119", name = "Blingtron 3000",
    text = "<b>Battlecry:</b> Equip a random weapon for each player.", set = "GVG",
    type = "minion", rarity = "legendary", cost = 5, attack = 3, health = 4,
    tags = { "mech" }, keywords = { "battlecry" },
}
local function weapons(ctx)
    local pool = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        if ctx:card_definition(id).type == "weapon" then pool[#pool + 1] = id end
    end
    return pool
end
function card.on_battlecry(ctx, self)
    local pool = weapons(ctx)
    if #pool > 0 then
        ctx:random_value(pool, "equip_owner")
        ctx:random_value(pool, "equip_opponent")
    end
end
function card.equip_owner(ctx, self, id) ctx:equip_weapon(ctx:controller(self), id) end
function card.equip_opponent(ctx, self, id) ctx:equip_weapon(ctx:opponent(ctx:controller(self)), id) end
return card
