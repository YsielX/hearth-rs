local card = {
    api_version = 1, id = "CFM_755", name = "Grimestreet Pawnbroker",
    text = "<b>Battlecry:</b> Give a random weapon in your hand +1/+1.", set = "GANGS",
    type = "minion", class = "warrior", rarity = "rare", cost = 3,
    attack = 3, health = 3, keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    local candidates = {}
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if ctx:entity(entity).type == "weapon" then candidates[#candidates + 1] = entity end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "buff_weapon") end
end
function card.buff_weapon(ctx, self, target) cardlib.effects.buff(ctx, target, 1, 1) end
return card
