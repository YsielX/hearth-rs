local card = {
    api_version = 1, id = "CFM_685", name = "Don Han'Cho",
    text = "<b>Battlecry:</b> Give a\nrandom minion in your hand +5/+5.", set = "GANGS",
    type = "minion", classes = { "hunter", "paladin", "warrior" }, rarity = "legendary",
    cost = 5, attack = 5, health = 5, keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    local candidates = {}
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if ctx:entity(entity).type == "minion" then candidates[#candidates + 1] = entity end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "buff_minion") end
end
function card.buff_minion(ctx, self, target) ctx:buff(target, 5, 5) end
return card
