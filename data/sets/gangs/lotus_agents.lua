local card = {
    api_version = 1, id = "CFM_852", name = "Lotus Agents",
    text = "<b>Battlecry:</b> <b>Discover</b>\na Druid, Rogue, or\nShaman card.",
    set = "GANGS", type = "minion", classes = { "druid", "rogue", "shaman" },
    rarity = "rare", cost = 3, attack = 3, health = 3,
    keywords = { "battlecry", "discover" },
}
local wanted = { druid = true, rogue = true, shaman = true }
local function eligible(definition)
    if wanted[definition.class] then return true end
    for _, class in ipairs(definition.classes or {}) do if wanted[class] then return true end end
    return false
end
function card.on_battlecry(ctx, self)
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        if eligible(ctx:card_definition(card_id)) then pool[#pool + 1] = card_id end
    end
    if #pool > 0 then ctx:discover_cards(ctx:controller(self), "Choose a Lotus card", pool, 3, "receive_card") end
end
function card.receive_card(ctx, self, card_id) ctx:give_card(ctx:controller(self), card_id) end
return card
