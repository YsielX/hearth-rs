local card = {
    api_version = 1, id = "CFM_853", name = "Grimestreet Smuggler",
    text = "<b>Battlecry:</b> Give a random minion in your hand +1/+1.", set = "GANGS",
    type = "minion", classes = { "hunter", "paladin", "warrior" }, rarity = "common",
    cost = 3, attack = 2, health = 4, keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    local candidates = {}
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if ctx:entity(entity).type == "minion" then candidates[#candidates + 1] = entity end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "buff_minion") end
end
function card.buff_minion(ctx, self, target) cardlib.effects.buff(ctx, target, 1, 1) end
return card
