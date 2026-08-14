local function murlocs(ctx)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        for _, tag in ipairs(ctx:card_definition(card_id).tags) do
            if tag == "murloc" or tag == "all" then
                result[#result + 1] = card_id
                break
            end
        end
    end
    return result
end

local card = {
    api_version = 1,
    id = "AT_076",
    name = "Murloc Knight",
    text = "<b>Inspire:</b> Summon a random Murloc.",
    set = "TGT",
    type = "minion",
    class = "paladin",
    rarity = "common",
    cost = 4,
    attack = 3,
    health = 4,
    tags = { "murloc" },
    keywords = { "inspire" },
}

function card.on_inspire(ctx, self)
    local pool = murlocs(ctx)
    if #pool > 0 then ctx:random_value(pool, "summon_murloc") end
end

function card.summon_murloc(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
