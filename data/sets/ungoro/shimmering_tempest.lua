local card = { api_version = 1, id = "UNG_846", name = "Shimmering Tempest",
    text = "<b>Battlecry:</b> Add a random Mage spell to your hand.", set = "UNGORO",
    type = "minion", class = "mage", rarity = "common", cost = 2, attack = 2, health = 2,
    tags = { "elemental" }, keywords = { "battlecry" } }
local function generatable(definition)
    for _, keyword in ipairs(definition.keywords or {}) do
        if keyword == "quest" or keyword == "questline" or keyword == "cannot_be_randomly_generated" then return false end
    end
    return true
end
function card.on_battlecry(ctx, self)
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        local mage = definition.class == "mage"
        for _, class in ipairs(definition.classes or {}) do if class == "mage" then mage = true end end
        if definition.type == "spell" and mage and generatable(definition) then pool[#pool + 1] = card_id end
    end
    if #pool > 0 then ctx:random_value(pool, "receive_spell") end
end
function card.receive_spell(ctx, self, card_id) cardlib.effects.give_card(ctx, ctx:controller(self), card_id) end
return card
