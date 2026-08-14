local card = {
    api_version = 1, id = "CFM_611", name = "Bloodfury Potion",
    text = "[x]Give a minion +3 Attack.\nIf it's a Demon, also\ngive it +3 Health.",
    set = "GANGS", type = "spell", class = "warlock", spell_school = "shadow",
    rarity = "rare", cost = 3, target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
}
local function demon(definition)
    for _, tag in ipairs(definition.tags or {}) do
        if tag == "demon" or tag == "all" then return true end
    end
    return false
end
function card.on_play(ctx, self, target)
    if demon(ctx:card_definition(ctx:entity(target).card_id)) then ctx:buff(target, 3, 3)
    else ctx:buff(target, 3, 0) end
end
return card
