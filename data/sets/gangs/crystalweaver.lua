local card = {
    api_version = 1, id = "CFM_610", name = "Crystalweaver",
    text = "<b>Battlecry:</b> Give your Demons +1/+1.", set = "GANGS", type = "minion",
    class = "warlock", rarity = "common", cost = 4, attack = 5, health = 4,
    tags = { "draenei" }, keywords = { "battlecry" },
}
local function demon(definition)
    for _, tag in ipairs(definition.tags or {}) do
        if tag == "demon" or tag == "all" then return true end
    end
    return false
end
function card.on_battlecry(ctx, self)
    for _, entity in ipairs(ctx:friendly_minions(self)) do
        if demon(ctx:card_definition(ctx:entity(entity).card_id)) then cardlib.effects.buff(ctx, entity, 1, 1) end
    end
end
return card
