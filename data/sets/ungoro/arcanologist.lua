local function secret(definition)
    for _, keyword in ipairs(definition.keywords or {}) do if keyword == "secret" then return true end end
    return false
end
local card = { api_version = 1, id = "UNG_020", name = "Arcanologist",
    text = "<b>Battlecry:</b> Draw a <b>Secret</b>.", set = "UNGORO", type = "minion",
    class = "mage", rarity = "common", cost = 2, attack = 2, health = 3, keywords = { "battlecry" } }
function card.on_battlecry(ctx, self)
    local candidates = {}
    for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
        if secret(ctx:card_definition(ctx:entity(entity).card_id)) then candidates[#candidates + 1] = entity end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "draw_secret") end
end
function card.draw_secret(ctx, self, entity) ctx:draw_entity(ctx:controller(self), entity) end
return card
