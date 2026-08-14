local function has_tag(definition, wanted)
    for _, tag in ipairs(definition.tags) do
        if tag == wanted then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "FP1_022",
    name = "Voidcaller",
    text = "<b>Deathrattle:</b> Put a random Demon from your hand into the battlefield.",
    set = "NAXX",
    type = "minion",
    class = "warlock",
    rarity = "common",
    cost = 4,
    attack = 3,
    health = 4,
    tags = { "demon" },
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local candidates = {}
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        local definition = ctx:card_definition(ctx:entity(entity).card_id)
        if definition.type == "minion" and has_tag(definition, "demon") then
            candidates[#candidates + 1] = entity
        end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "summon_demon") end
end

function card.summon_demon(ctx, self, entity)
    ctx:summon_from_hand(entity)
end

return card
