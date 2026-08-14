local function deck_minions(ctx, player)
    local result = {}
    for _, entity in ipairs(ctx:deck(player)) do
        if ctx:entity(entity).type == "minion" then result[#result + 1] = entity end
    end
    return result
end

local card = {
    api_version = 1, id = "AT_128", name = "The Skeleton Knight",
    text = "<b>Deathrattle:</b> Reveal a minion in each deck. If yours costs more, return this to your hand.",
    set = "TGT", type = "minion", rarity = "legendary", cost = 6, attack = 7, health = 4,
    tags = { "undead" }, keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local candidates = deck_minions(ctx, ctx:controller(self))
    if #candidates > 0 then ctx:random_value(candidates, "reveal_friendly_minion") end
end

function card.reveal_friendly_minion(ctx, self, entity)
    ctx:set_data(self, "friendly_cost", ctx:entity(entity).cost)
    local candidates = deck_minions(ctx, ctx:opponent(ctx:controller(self)))
    if #candidates > 0 then ctx:random_value(candidates, "reveal_enemy_minion")
    else ctx:move(self, "hand") end
end

function card.reveal_enemy_minion(ctx, self, entity)
    if ctx:get_data(self, "friendly_cost") > ctx:entity(entity).cost then ctx:move(self, "hand") end
end

return card
