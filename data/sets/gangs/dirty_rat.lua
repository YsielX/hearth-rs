local card = {
    api_version = 1, id = "CFM_790", name = "Dirty Rat",
    text = "[x]<b>Taunt</b>\n<b>Battlecry:</b> Your opponent\nsummons a random minion\nfrom their hand.",
    set = "GANGS", type = "minion", rarity = "epic", cost = 2, attack = 2,
    health = 6, keywords = { "taunt", "battlecry" },
}
function card.on_battlecry(ctx, self)
    local opponent = ctx:opponent(ctx:controller(self))
    local candidates = {}
    for _, entity in ipairs(ctx:hand(opponent)) do
        if ctx:entity(entity).type == "minion" then candidates[#candidates + 1] = entity end
    end
    if #candidates > 0 then ctx:random_value(candidates, "summon_minion") end
end
function card.summon_minion(ctx, self, entity) ctx:summon_from_hand(entity) end
return card
