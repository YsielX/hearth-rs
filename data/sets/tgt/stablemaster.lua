local card = {
    api_version = 1, id = "AT_057", name = "Stablemaster",
    text = "<b>Battlecry:</b> Give a friendly Beast <b>Immune</b> this turn.",
    set = "TGT", type = "minion", class = "hunter", rarity = "epic",
    cost = 3, attack = 4, health = 2, keywords = { "battlecry" },
    target_mode = "required_if_available",
}

local function has_beast_tag(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags) do
        if tag == "beast" or tag == "all" then return true end
    end
    return false
end

function card.targets(ctx, self)
    local result = {}
    for _, minion in ipairs(ctx:board(ctx:controller(self))) do
        if has_beast_tag(ctx, minion) then result[#result + 1] = minion end
    end
    return result
end

function card.on_battlecry(ctx, self, target)
    if target then cardlib.effects.grant_keyword_until_end_of_turn(ctx, target, "immune") end
end

return card
