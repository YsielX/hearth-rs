local function has_tag(definition, wanted)
    for _, tag in ipairs(definition.tags) do
        if tag == wanted then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "FP1_011",
    name = "Webspinner",
    text = "<b>Deathrattle:</b> Get a\nrandom Beast.",
    set = "NAXX",
    type = "minion",
    class = "hunter",
    rarity = "common",
    cost = 1,
    attack = 1,
    health = 1,
    tags = { "beast" },
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local candidates = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        if has_tag(ctx:card_definition(card_id), "beast") then
            candidates[#candidates + 1] = card_id
        end
    end
    if #candidates > 0 then ctx:random_value(candidates, "give_beast") end
end

function card.give_beast(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

return card
