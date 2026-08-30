local function eligible(ctx, player, definition)
    local own = ctx:player(player).class
    if definition.class == "neutral" or definition.class == own then return true end
    for _, class in ipairs(definition.classes or {}) do if class == own then return true end end
    return false
end
local card = { api_version = 1, id = "UNG_078", name = "Tortollan Forager",
    text = "<b>Battlecry:</b> Add a random minion with 5 or more Attack to your hand.",
    set = "UNGORO", type = "minion", class = "druid", rarity = "common",
    cost = 2, attack = 2, health = 2, keywords = { "battlecry" } }
function card.on_battlecry(ctx, self)
    local player, pool = ctx:controller(self), {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.attack >= 5 and eligible(ctx, player, definition) then
            pool[#pool + 1] = card_id
        end
    end
    if #pool > 0 then ctx:random_value(pool, "receive_minion") end
end
function card.receive_minion(ctx, self, card_id) cardlib.effects.give_card(ctx, ctx:controller(self), card_id) end
return card
