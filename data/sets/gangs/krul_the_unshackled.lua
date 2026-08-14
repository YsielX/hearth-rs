local card = {
    api_version = 1, id = "CFM_750", name = "Krul the Unshackled",
    text = "[x]<b>Battlecry:</b> If your deck has\nno duplicates, summon all\n Demons from your hand. ",
    set = "GANGS", type = "minion", class = "warlock", rarity = "legendary",
    cost = 9, attack = 9, health = 9, tags = { "demon" }, keywords = { "battlecry" },
}
local function demon(definition)
    for _, tag in ipairs(definition.tags or {}) do
        if tag == "demon" or tag == "all" then return true end
    end
    return false
end
function card.on_battlecry(ctx, self)
    local player, seen = ctx:controller(self), {}
    for _, entity in ipairs(ctx:deck(player)) do
        local card_id = ctx:entity(entity).card_id
        if seen[card_id] then return end
        seen[card_id] = true
    end
    for _, entity in ipairs(ctx:hand(player)) do
        if demon(ctx:card_definition(ctx:entity(entity).card_id)) then ctx:summon_from_hand(entity) end
    end
end
return card
