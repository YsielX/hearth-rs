local card = {
    api_version = 1, id = "OG_048", name = "Mark of Y'Shaarj",
    text = "Give a minion +2/+2.\nIf it's a Beast, draw\na card.",
    set = "OG", type = "spell", class = "druid", rarity = "common", cost = 2,
    target_mode = "required", targets = function(ctx) return ctx:minions() end,
}

local function beast(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags) do
        if tag == "beast" or tag == "all" then return true end
    end
    return false
end

function card.on_play(ctx, self, target)
    local draw = beast(ctx, target)
    cardlib.effects.buff(ctx, target, 2, 2)
    if draw then ctx:draw(ctx:controller(self), 1) end
end
return card
