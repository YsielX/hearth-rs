local card = {
    api_version = 1, id = "CFM_650", name = "Grimscale Chum",
    text = "[x]<b>Battlecry:</b> Give a random\nMurloc in your hand +1/+1.",
    set = "GANGS", type = "minion", class = "paladin", rarity = "common",
    cost = 1, attack = 2, health = 1, tags = { "murloc" }, keywords = { "battlecry" },
}
local function is_murloc(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "murloc" or tag == "all" then return true end
    end
    return false
end
function card.on_battlecry(ctx, self)
    local candidates = {}
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if ctx:entity(entity).type == "minion" and is_murloc(ctx, entity) then candidates[#candidates + 1] = entity end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "buff_grimscale_murloc") end
end
function card.buff_grimscale_murloc(ctx, self, target) cardlib.effects.buff(ctx, target, 1, 1) end
return card
