local function eligible(ctx, player, definition)
    local own = ctx:player(player).class
    if definition.class == "neutral" or definition.class == own then return true end
    for _, class in ipairs(definition.classes or {}) do if class == own then return true end end
    return false
end
local function beast(definition)
    for _, tag in ipairs(definition.tags or {}) do if tag == "beast" or tag == "all" then return true end end
    return false
end
local card = { api_version = 1, id = "UNG_912", name = "Jeweled Macaw",
    text = "<b>Battlecry:</b> Add a random Beast to your hand.", set = "UNGORO", type = "minion",
    class = "hunter", rarity = "common", cost = 1, attack = 1, health = 2,
    tags = { "beast" }, keywords = { "battlecry" } }
function card.on_battlecry(ctx, self)
    local player, pool = ctx:controller(self), {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and beast(definition) and eligible(ctx, player, definition) then pool[#pool + 1] = card_id end
    end
    if #pool > 0 then ctx:random_value(pool, "receive_beast") end
end
function card.receive_beast(ctx, self, card_id) ctx:give_card(ctx:controller(self), card_id) end
return card
