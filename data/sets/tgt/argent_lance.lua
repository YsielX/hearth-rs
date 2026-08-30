local function deck_minions(ctx, player)
    local result = {}
    for _, entity in ipairs(ctx:deck(player)) do
        if ctx:entity(entity).type == "minion" then result[#result + 1] = entity end
    end
    return result
end

local card = {
    api_version = 1,
    id = "AT_077",
    name = "Argent Lance",
    text = "<b>Battlecry:</b> Reveal a minion in each deck. If yours costs more, +1 Durability.",
    set = "TGT",
    type = "weapon",
    class = "paladin",
    rarity = "rare",
    cost = 2,
    attack = 2,
    health = 2,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local own = deck_minions(ctx, player)
    if #own == 0 then return end
    if #deck_minions(ctx, ctx:opponent(player)) == 0 then
        cardlib.effects.buff(ctx, self, 0, 1)
        return
    end
    ctx:random_value(own, "reveal_friendly_minion")
end

function card.reveal_friendly_minion(ctx, self, entity)
    ctx:set_data(self, "joust_cost", ctx:entity(entity).cost)
    local opponent = ctx:opponent(ctx:controller(self))
    ctx:random_value(deck_minions(ctx, opponent), "reveal_enemy_minion")
end

function card.reveal_enemy_minion(ctx, self, entity)
    if ctx:get_data(self, "joust_cost") > ctx:entity(entity).cost then cardlib.effects.buff(ctx, self, 0, 1) end
end

return card
