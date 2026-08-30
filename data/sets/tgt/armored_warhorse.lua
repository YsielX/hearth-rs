local function deck_minions(ctx, player)
    local result = {}
    for _, entity in ipairs(ctx:deck(player)) do
        if ctx:entity(entity).type == "minion" then result[#result + 1] = entity end
    end
    return result
end

local card = {
    api_version = 1,
    id = "AT_108",
    name = "Armored Warhorse",
    text = "<b>Battlecry:</b> Reveal a minion in each deck. If yours costs more, gain <b>Charge</b>.",
    set = "TGT",
    type = "minion",
    rarity = "rare",
    cost = 4,
    attack = 5,
    health = 3,
    tags = { "beast" },
    keywords = { "battlecry" },
}

local function win(ctx, self) cardlib.effects.grant_keyword(ctx, self, "charge") end

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local own = deck_minions(ctx, player)
    if #own == 0 then return end
    if #deck_minions(ctx, ctx:opponent(player)) == 0 then
        win(ctx, self)
        return
    end
    ctx:random_value(own, "reveal_friendly_minion")
end

function card.reveal_friendly_minion(ctx, self, entity)
    ctx:set_data(self, "joust_cost", ctx:entity(entity).cost)
    ctx:random_value(deck_minions(ctx, ctx:opponent(ctx:controller(self))), "reveal_enemy_minion")
end

function card.reveal_enemy_minion(ctx, self, entity)
    if ctx:get_data(self, "joust_cost") > ctx:entity(entity).cost then win(ctx, self) end
end

return card
