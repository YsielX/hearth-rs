local function deck_minions(ctx, player)
    local result = {}
    for _, entity in ipairs(ctx:deck(player)) do
        if ctx:entity(entity).type == "minion" then result[#result + 1] = entity end
    end
    return result
end

local card = {
    api_version = 1, id = "AT_133", name = "Gadgetzan Jouster",
    text = "<b>Battlecry:</b> Reveal a minion in each deck. If yours costs more, gain +1/+1.",
    set = "TGT", type = "minion", rarity = "common", cost = 1, attack = 1, health = 2,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local candidates = deck_minions(ctx, ctx:controller(self))
    if #candidates > 0 then ctx:random_value(candidates, "reveal_friendly_minion") end
end

function card.reveal_friendly_minion(ctx, self, entity)
    ctx:set_data(self, "friendly_cost", ctx:entity(entity).cost)
    local candidates = deck_minions(ctx, ctx:opponent(ctx:controller(self)))
    if #candidates > 0 then ctx:random_value(candidates, "reveal_enemy_minion")
    else ctx:buff(self, 1, 1) end
end

function card.reveal_enemy_minion(ctx, self, entity)
    if ctx:get_data(self, "friendly_cost") > ctx:entity(entity).cost then ctx:buff(self, 1, 1) end
end

return card
