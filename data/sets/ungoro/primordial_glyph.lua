local card = { api_version = 1, id = "UNG_941", name = "Primordial Glyph",
    text = "<b>Discover</b> a spell. Reduce its Cost by (2).", set = "UNGORO",
    type = "spell", class = "mage", rarity = "epic", spell_school = "arcane", cost = 2,
    keywords = { "discover" }, triggers = {{ event = "card_created", timing = "after", active_zones = { "graveyard" },
        condition = function(ctx, self, event) return event.source == self end,
        effect = function(ctx, self, event) cardlib.effects.modify(ctx, event.entity, { stat = "cost", operation = "add", value = -2 }) end }} }
local function generatable(definition)
    for _, keyword in ipairs(definition.keywords or {}) do
        if keyword == "quest" or keyword == "questline" or keyword == "cannot_be_randomly_generated" then return false end
    end
    return true
end
function card.on_play(ctx, self)
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        local eligible = definition.class == "mage" or definition.class == "neutral"
        for _, class in ipairs(definition.classes or {}) do if class == "mage" then eligible = true end end
        if definition.type == "spell" and eligible and generatable(definition) then pool[#pool + 1] = card_id end
    end
    if #pool > 0 then ctx:discover_cards(ctx:controller(self), "Discover a spell", pool, 3, "receive_spell") end
end
function card.receive_spell(ctx, self, card_id) cardlib.effects.give_card(ctx, ctx:controller(self), card_id) end
return card
